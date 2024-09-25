import random
from typing import Iterator

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
) -> list[float]:
    detector = helpers.PeakDetector()
    notes = list(note_provider)
    with detector.wrap(audio_interface):
        for _ in range(5):
            for m_note, d_note in notes:
                left.play_note(m_note)
                right.play_note(d_note)
                helpers.sleep(100)

                left.play_note(d_note, False)
                right.play_note(m_note, False)
                helpers.sleep(700)
        helpers.sleep(2500)

    left_delays, right_delays = detector.peaks
    lens = len(left_delays), len(right_delays), len(notes) * 5
    if not (lens[0] == lens[1] == lens[2]):
        print(f"Test failed due to missing elements: {lens}")
        exit(-1)
    return [abs(left_ts - right_ts) for left_ts, right_ts in zip(left_delays, right_delays)]


def note_factory(base: int) -> Iterator[tuple[int, int]]:
    seq = list(range(base, base+13))
    random.shuffle(seq)

    for elem in seq:
        yield elem, 50  # high hat


no_sync = test_latency_differences(mechbass_unsynced, drumbot_unsynced, note_factory(43))
sync = test_latency_differences(mechbass, drumbot, note_factory(43))

print(sync)
print(no_sync)