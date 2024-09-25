import contextlib
import functools
import time
from typing import Protocol, Callable, Iterator

import sounddevice as sd
import numpy as np
import mido

sleep = sd.sleep


class AudioContext(Protocol):
    def on_buffer(self, left: np.ndarray, right: np.ndarray):
        ...

    @contextlib.contextmanager
    def wrap(self, source: "AudioInput"):
        with source.record(self) as _:
            yield


class AudioInput:
    def __init__(self, device_name: str, samplerate: int = 44100):
        self.device_name = device_name
        self.samplerate = samplerate

    def on_buffer(self, ctx: AudioContext, indata: np.ndarray, *rest):
        # split ndarray into left and right channels
        left, right = np.hsplit(indata, 2)  # type: np.ndarray
        ctx.on_buffer(np.ravel(left), np.ravel(right))

    @contextlib.contextmanager
    def record(self, ctx: AudioContext):
        with sd.Stream(
                samplerate=self.samplerate,
                callback=functools.partial(self.on_buffer, ctx),
                device=self.device_name
        ):
            yield


class PeakDetector(AudioContext):
    class PeakState:
        def __init__(self):
            self.peaks: list[float] = []
            self.triggered = False
            self.last_peak = 0.0

    def __init__(
            self,
            chunks: int = 16,
            samplerate: int = 44100,
            sensitivity: float = 1.25,
            peak_duration: float = 0.050
    ):
        self.left_peaks = PeakDetector.PeakState()
        self.right_peaks = PeakDetector.PeakState()

        self.chunks: int = chunks
        self.sample_rate: int = samplerate

        self.sensitivity = sensitivity
        self.peak_duration = peak_duration

    def on_buffer(self, left: np.ndarray, right: np.ndarray):
        self.find_peaks(left, self.left_peaks)
        self.find_peaks(right, self.right_peaks)

    def find_peaks(self, buffer: np.ndarray, state: PeakState):
        # chunk data into batches
        ts = time.time()
        chunks = np.array_split(buffer, self.chunks)

        for i, chunk in enumerate(reversed(chunks)):
            # find the timestamp based on how far back in the buffer it is
            ts_offset = ts - (i + 1) * (self.chunks / self.sample_rate)

            # too early for a new peak, likely noise
            if ts_offset < state.last_peak + self.peak_duration:
                continue

            volume = np.linalg.norm(chunk)

            # volume is not a peak
            if volume < self.sensitivity:
                state.triggered = False
                state.last_peak = False
                continue

            # we've already looking at a peak
            if state.triggered:
                continue

            # register a new peak
            state.triggered = True
            state.last_peak = ts_offset
            state.peaks.append(ts_offset)

    @property
    def peaks(self) -> tuple[list[float], list[float]]:
        return self.left_peaks.peaks, self.right_peaks.peaks


class MidiPlayer:
    def __init__(self, port_name: str):
        self.notes: list[tuple[mido.Message, float]] = []
        self.port = mido.open_output(port_name)

    def play_note(self, note: int, play: bool = True, channel: int = 0, velocity: int = 127):
        ts = time.time()
        msg = mido.Message("note_on" if play else "note_off", note=note, channel=channel, velocity=velocity)
        self.port.send(msg)
        if play:
            self.notes.append((msg, ts))
