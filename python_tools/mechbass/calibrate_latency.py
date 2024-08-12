import csv
import itertools
import time
from typing import Callable, NamedTuple

import sounddevice as sd
import numpy as np
from mido import open_output, Message

TUNING = (43, 38, 33, 28)
FRETS = 13
MECHBASS_PORT = "Scarlett 18i20 USB:Scarlett 18i20 USB MIDI 1"
MICROPHONE_DEVICE = "Scarlett 2i2 USB"
SAMPLE_RATE = 44100
CHUNKS = 16
TS_OFFSET = CHUNKS / SAMPLE_RATE
MAX_PAN_TIME = 600
ORACLE_TIME = 500
SAMPLES = 5


def sys_sleep(ms: int):
    time.sleep(ms / 1000)


def play_note(
        port,
        note: int,
        duration_ms: int = 100,
        channel: int = 0,
        sleep: Callable[[int], None] = sys_sleep
):
    port.send(Message("note_on", note=note, channel=channel, velocity=127))
    sleep(duration_ms)
    port.send(Message("note_off", note=note, channel=channel))


class ImpulseHandler:
    def __init__(self):
        self.sensitivity = 2.5
        self.peak_delay = 0.020  # 20ms
        self.triggered = False
        self.last_peak = 0.0
        self.callback: Callable[[float], None] | None = None

    def __call__(self, indata: np.ndarray, *_rest):
        if not self.callback:
            return
        ts = time.time()
        chunks = np.array_split(indata, CHUNKS)
        for i, chunk in enumerate(reversed(chunks)):
            ts_offset = ts - (i + 1) * TS_OFFSET
            if ts_offset < self.last_peak + self.peak_delay:
                return
            volume = np.linalg.norm(chunk)
            if volume < self.sensitivity:
                self.triggered = False
                continue
            if self.triggered:
                continue
            self.triggered = True
            self.last_peak = ts_offset
            self.callback(ts_offset)


port = open_output(MECHBASS_PORT)
handler = ImpulseHandler()


def test_latency(handler: ImpulseHandler, first: int, second: int, channel: int) -> float:
    play_note(port, first, duration_ms=0, sleep=sd.sleep, channel=channel)
    sd.sleep(MAX_PAN_TIME)

    notes = []
    handler.callback = notes.append

    play_note(port, first, duration_ms=0, sleep=sd.sleep, channel=channel)
    sd.sleep(ORACLE_TIME)
    play_note(port, second, duration_ms=0, sleep=sd.sleep, channel=channel)

    sd.sleep(MAX_PAN_TIME)
    if len(notes) != 2:
        raise Exception(f"Buffer does not contain 2 notes, {notes} found")
    return (notes[1] - notes[0]) - (ORACLE_TIME / 1000)


class Measurement(NamedTuple):
    first: int
    second: int
    latencies: list[float]


def dist(lowest: int, first: int, second: int) -> float:
    d_first = 2 ** ((lowest - first) / 12)
    d_second = 2 ** ((lowest - second) / 12)

    return abs(d_first - d_second)


def np_str(f: float) -> str:
    return np.format_float_positional(f, trim='0')


errors = []
with sd.Stream(samplerate=44100, callback=handler, device=MICROPHONE_DEVICE):
    channel = 0
    lowest = TUNING[channel]
    perms = sorted(set(itertools.product(range(lowest, lowest + FRETS + 1), repeat=2)))
    test_count = len(perms)
    duration_mins = (MAX_PAN_TIME * 2 + ORACLE_TIME) * SAMPLES * test_count / 1000 / 60
    print(f"Performing {test_count} latency tests")
    print(f"This calibration will take {duration_mins:.2f} minutes")

    with open(f"channel_{channel}_{lowest}_{lowest + FRETS}.csv", "w+") as csv_ptr:
        writer = csv.writer(csv_ptr)
        writer.writerow(["first", "second", *(f"trial_{i}_time" for i in range(SAMPLES)), "distance", "average_time"])
        for itt, (first, second) in enumerate(reversed(perms)):
            try:
                percentage = itt / test_count
                print(f"{percentage * 100:.2f}% Complete, {(1 - percentage) * duration_mins:.2f} minutes remain")
                print(f"Testing {first} -> {second}:")
                latency_acc = []

                for i in range(SAMPLES):
                    latency = test_latency(handler, first, second, channel)
                    print(f"    {i + 1}: {latency * 1000:.2f}ms")
                    latency_acc.append(latency)

                distance = dist(lowest, first, second)
                avg_latency = float(np.mean(latency_acc))
                print(f"Average: {avg_latency * 1000:.2f}ms | {distance:.4f} string")
                print()

                writer.writerow([first, second, *(np_str(f) for f in latency_acc), np_str(distance), np_str(avg_latency)])
            except Exception as ex:
                print(f"AN ERROR OCCURED: {ex}")
                errors.append((first, second, ex))

if errors:
    print(f"The following {len(errors)} errors occured:")
    for first, second, ex in errors:
        print(f"{first} -> {second}: {ex}")
else:
    print("No errors occured :)")
