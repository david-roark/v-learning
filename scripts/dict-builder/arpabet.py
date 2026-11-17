"""ARPAbet (CMUdict) to IPA converter with primary/secondary stress support."""

from __future__ import annotations

ARPABET_TO_IPA: dict[str, str] = {
    "AA": "ɑ",
    "AE": "æ",
    "AH": "ʌ",
    "AO": "ɔ",
    "AW": "aʊ",
    "AY": "aɪ",
    "B": "b",
    "CH": "tʃ",
    "D": "d",
    "DH": "ð",
    "EH": "ɛ",
    "ER": "ɝ",
    "EY": "eɪ",
    "F": "f",
    "G": "ɡ",
    "HH": "h",
    "IH": "ɪ",
    "IY": "i",
    "JH": "dʒ",
    "K": "k",
    "L": "l",
    "M": "m",
    "N": "n",
    "NG": "ŋ",
    "OW": "oʊ",
    "OY": "ɔɪ",
    "P": "p",
    "R": "ɹ",
    "S": "s",
    "SH": "ʃ",
    "T": "t",
    "TH": "θ",
    "UH": "ʊ",
    "UW": "u",
    "V": "v",
    "W": "w",
    "Y": "j",
    "Z": "z",
    "ZH": "ʒ",
}


def arpabet_to_ipa(phonemes: str) -> str:
    """Convert space-separated ARPAbet tokens (e.g. 'R AH1 N') to '/rʌn/'."""
    tokens = phonemes.strip().split()
    if not tokens:
        return ""

    ipa_parts: list[str] = []
    for tok in tokens:
        stress = ""
        base = tok
        if tok[-1].isdigit():
            digit = tok[-1]
            base = tok[:-1]
            if digit == "1":
                stress = "ˈ"
            elif digit == "2":
                stress = "ˌ"
            # Unstressed AH0 maps to schwa
            if digit == "0" and base == "AH":
                ipa_parts.append("ə")
                continue
            if digit == "0" and base == "ER":
                ipa_parts.append("ɚ")
                continue

        mapped = ARPABET_TO_IPA.get(base)
        if not mapped:
            continue
        if stress:
            ipa_parts.append(f"{stress}{mapped}")
        else:
            ipa_parts.append(mapped)

    return f"/{''.join(ipa_parts)}/" if ipa_parts else ""
