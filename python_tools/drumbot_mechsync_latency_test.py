import itertools
import random
import time
from typing import Iterator, List, Tuple, Any

import helpers

# ensure that the required midi devices are available
try:
    drumbot = helpers.MidiPlayer("MechSync:DrumBot Input")
    mechbass = helpers.MidiPlayer("MechSync:MechBass Input")
    drumbot_unsynced = helpers.MidiPlayer("DrumBot:DrumBot Port 1")
    mechbass_unsynced = helpers.MidiPlayer("Scarlett 18i20 USB:Scarlett 18i20 USB MIDI 1")
except OSError as ex:
    print(f"Unable to bind midi device: {ex} - Did you make sure that the device has been plugged in?")
    exit(-1)

audio_interface = helpers.AudioInput("Scarlett 2i2 USB")


def test_latency_differences(
        left: helpers.MidiPlayer,
        right: helpers.MidiPlayer,
        note_provider: Iterator[tuple[int, int]] = iter(())
) -> list[tuple[float, float]]:
    detector = helpers.PeakDetector()
    notes = list(note_provider)
    oracle = []
    with detector.wrap(audio_interface):
        for _ in range(5):
            for m_note, d_note in notes:
                left.play_note(m_note)
                right.play_note(d_note)
                oracle.append(time.time())
                helpers.sleep(100)

                left.play_note(d_note, False)
                right.play_note(m_note, False)
                helpers.sleep(700)
        helpers.sleep(2500)

    left_delays, right_delays = detector.peaks
    left_len, right_len, oracle_len = len(left_delays), len(right_delays), len(oracle)
    if not (left_len == right_len == oracle_len):
        print(f"Test failed due to missing elements: left:{left_len} right:{right_len} oracle:{oracle_len}")
        exit(-1)
    return [(left_ts - oracle_ts, right_ts - oracle_ts) for left_ts, right_ts, oracle_ts in zip(left_delays, right_delays, oracle)]


def note_factory(base: int) -> Iterator[tuple[int, int]]:
    yield base, 50
    for i in range(14):
        yield base, 50
        yield base + i, 50

def display(values: list[tuple[float, float]]):
    data: list = list(zip(*itertools.batched(values, 14*2+1)))
    for row in data:
        print(f"{", ".join(str(e[0]) for e in row)}, {", ".join(str(e[1]) for e in row)}")
    print("-----------------")


sync = test_latency_differences(mechbass, drumbot, note_factory(43))
display(sync)
no_sync = test_latency_differences(mechbass_unsynced, drumbot_unsynced, note_factory(43))
display(no_sync)
