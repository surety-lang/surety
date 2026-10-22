#!/usr/bin/env python3
"""The optical recall cache, measured (surety-docs/Experiment — optical recall cache).

Prices the corpus as text at four levels of lossy projection, renders the same text
as images, and prices those under Anthropic's documented image rule, so the question
"does rendering obligations as images save tokens" has a number instead of a hope.

    python3 experiments/optical_cache.py <out-dir>              # tables on stdout, PNGs in out-dir
    python3 experiments/optical_cache.py --score truth.txt got.txt   # token error rate of a read-back

Text tokens are tiktoken cl100k_base, a proxy: Claude's tokenizer counts about 10-20%
more on code-like text, which flatters the image side. Image tokens follow the rule
on platform.claude.com/docs/en/build-with-claude/vision: the image is scaled to fit
the tier's long edge and token cap, then costs ceil(w/28) * ceil(h/28). Two tiers:
standard (1568 px, 1568 tokens) and high resolution (2576 px, 4784 tokens; Opus 4.7
and later per the docs). The blind read-back (fresh agents given only the PNG) is
run by hand; --score grades a transcription against the source.
"""
from __future__ import annotations

import difflib
import json
import math
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORPUS = ROOT / "crates" / "surety" / "tests" / "corpus"
ORDER = ["p1", "p2", "p3", "p4", "p5a", "p5b", "p6", "p7", "p8", "apply_diff", "p14"]
FONT = "/System/Library/Fonts/Menlo.ttc"
TIERS = {"std": (1568, 1568), "hi": (2576, 4784)}


def image_tokens(w: int, h: int, tier: str) -> tuple[int, float]:
    """(tokens, scale) for a w x h image under a tier: shrink to the long edge, then to the token cap."""
    edge, cap = TIERS[tier]
    s = min(1.0, edge / max(w, h))
    while math.ceil(w * s / 28) * math.ceil(h * s / 28) > cap:
        s *= 0.995
    return math.ceil(w * s / 28) * math.ceil(h * s / 28), s


def wrap(line: str, width: int) -> list[str]:
    if len(line) <= width:
        return [line]
    out, indent = [], len(line) - len(line.lstrip("\t "))
    pad = " " * min(indent + 2, width // 2)
    while len(line) > width:
        cut = line.rfind(" ", 0, width)
        if cut <= len(pad):
            cut = width
        out.append(line[:cut])
        line = pad + line[cut:].lstrip()
    out.append(line)
    return out


def layout(blocks: list[tuple[str | None, str]], col_chars: int) -> list[tuple[str, bool]]:
    lines: list[tuple[str, bool]] = []
    for title, text in blocks:
        if title:
            lines.append((f"=== {title} ===", True))
        for l in text.expandtabs(4).splitlines():
            lines.extend((w, False) for w in wrap(l, col_chars))
        lines.append(("", False))
    return lines


def render(blocks, px: int, col_chars: int = 64, max_cols: int = 12, tier: str = "std"):
    """One image of the blocks at font size px, with the column count that the tier downscales least.
    Returns (image, dict of natural size, columns, tokens, scale, effective font size)."""
    from PIL import Image, ImageDraw, ImageFont

    font = ImageFont.truetype(FONT, px, index=0)
    cw, lh = font.getlength("M"), round(px * 1.25)
    lines = layout(blocks, col_chars)
    best = None
    for cols in range(1, max_cols + 1):
        rows = math.ceil(len(lines) / cols)
        w, h = int(cols * (col_chars + 3) * cw) + 2 * px, rows * lh + 2 * px
        t, s = image_tokens(w, h, tier)
        if best is None or s > best["scale"] or (s == best["scale"] and t < best["tokens"]):
            best = dict(w=w, h=h, cols=cols, rows=rows, tokens=t, scale=s, eff_px=px * s)
    img = Image.new("RGB", (best["w"], best["h"]), "white")
    d = ImageDraw.Draw(img)
    for i, (l, head) in enumerate(lines):
        c, r = divmod(i, best["rows"])
        d.text((px + c * (col_chars + 3) * cw, px + r * lh), l, font=font, fill=(120, 0, 0) if head else (0, 0, 0))
    return img, best


def sheets(blocks, px: int, col_chars: int = 64, tier: str = "std"):
    """As many images as it takes to hold the blocks at scale 1.0 under the tier (legible as rendered)."""
    from PIL import Image, ImageDraw, ImageFont

    edge, cap = TIERS[tier]
    font = ImageFont.truetype(FONT, px, index=0)
    cw, lh = font.getlength("M"), round(px * 1.25)
    colw = (col_chars + 3) * cw
    cols = max(1, int(edge // colw))
    w = int(cols * colw) + 2 * px
    h = min(edge, int(cap * 784 / w))
    rows_per = (h - 2 * px) // lh
    lines = layout(blocks, col_chars)
    per = rows_per * cols
    imgs = []
    for k in range(math.ceil(len(lines) / per)):
        chunk = lines[k * per:(k + 1) * per]
        rr = min(rows_per, math.ceil(len(chunk) / cols))
        img = Image.new("RGB", (w, rr * lh + 2 * px), "white")
        d = ImageDraw.Draw(img)
        for i, (l, head) in enumerate(chunk):
            c, r = divmod(i, rr)
            d.text((px + c * colw, px + r * lh), l, font=font, fill=(120, 0, 0) if head else (0, 0, 0))
        imgs.append(img)
    return imgs


def projections(text: str):
    """The text ladder: full, without examples (cut at the first `e:`), signature + pinned sentences, index line."""
    lines = text.splitlines()
    first_e = next((i for i, l in enumerate(lines) if re.match(r"^\s*e:\s*$", l)), len(lines))
    sig = next(l for l in lines if re.match(r"^[a-z_]+\(.*\)\s*->", l))
    body = lines[:first_e]
    props = [l for l in body if re.match(r"^\s+(all|any|none|exactly_one|out|unique|len|sum)\b", l)]
    n_ex = sum(1 for l in lines if re.match(r"^\s*e:\s*$", l))
    sentences = [l.strip().lstrip("# ").strip() for l in body if re.match(r"^\s*#", l)]
    return dict(
        name=sig.split("(")[0],
        full=text,
        no_examples="\n".join(body).rstrip() + "\n",
        sentences="\n".join([sig] + sentences) + "\n",
        index=f"{sig}  # {len(props)} props, {n_ex} ex, verified",
    )


def score(truth_path: str, got_path: str) -> None:
    truth = pathlib.Path(truth_path).read_text().split()
    got = re.sub(r"^```\w*\n|\n```\s*$", "", pathlib.Path(got_path).read_text().strip()).split()
    sm = difflib.SequenceMatcher(None, truth, got, autojunk=False)
    errs, n = [], 0
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag != "equal":
            n += max(i2 - i1, j2 - j1)
            errs.append(f"{' '.join(truth[i1:i2]) or '∅'} -> {' '.join(got[j1:j2]) or '∅'}")
    print(f"token error rate {100 * n / len(truth):.1f}% ({n} of {len(truth)} tokens); {'; '.join(errs) or 'no errors'}")


def main(out: pathlib.Path) -> None:
    import tiktoken

    enc = tiktoken.get_encoding("cl100k_base")
    tok = lambda s: len(enc.encode(s))
    out.mkdir(parents=True, exist_ok=True)
    P = [projections((CORPUS / f"{n}.sur").read_text()) for n in ORDER]
    perf = {n: tok((CORPUS / f"{n}.py").read_text()) for n in ORDER}

    print("text tokens (cl100k) per projection, and the Python performance for comparison")
    print(f"{'obligation':32} {'full':>6} {'noExmpl':>8} {'sig+sent':>9} {'index':>6} {'perf':>6}")
    tot = dict(full=0, no_examples=0, sentences=0, index=0, perf=0)
    for n, p in zip(ORDER, P):
        row = {k: tok(p[k]) for k in ("full", "no_examples", "sentences", "index")}
        row["perf"] = perf[n]
        for k in tot:
            tot[k] += row[k]
        print(f"{p['name']:32} {row['full']:6} {row['no_examples']:8} {row['sentences']:9} {row['index']:6} {row['perf']:6}")
    print(f"{'TOTAL':32} {tot['full']:6} {tot['no_examples']:8} {tot['sentences']:9} {tot['index']:6} {tot['perf']:6}")
    print(f"{'ratio to full':32} {1:6.1f} {tot['full'] / tot['no_examples']:8.1f} {tot['full'] / tot['sentences']:9.1f} {tot['full'] / tot['index']:6.1f}")
    (out / "index.txt").write_text("\n".join(p["index"] for p in P) + "\n")
    (out / "sentences.txt").write_text("\n".join(p["sentences"] for p in P))
    (out / "no_examples.txt").write_text("\n\n".join(p["no_examples"] for p in P))

    blocks = [(p["name"], p["full"]) for p in P]
    print("\nthe whole corpus as ONE image, by font size and tier (effPx = font size after the tier's downscale)")
    print(f"{'px':>3} {'natural':>10} | {'std tok':>7} {'effPx':>5} {'ratio':>5} | {'hi tok':>6} {'effPx':>5} {'ratio':>5}")
    for px in (6, 7, 8, 9, 11, 14):
        img, b = render(blocks, px, tier="hi")
        img.save(out / f"corpus_one_{px}px.png")
        ts, ss = image_tokens(b["w"], b["h"], "std")
        th, sh = image_tokens(b["w"], b["h"], "hi")
        print(f"{px:3} {b['w']:>5}x{b['h']:<4} | {ts:7} {px * ss:5.1f} {tot['full'] / ts:5.2f} | {th:6} {px * sh:5.1f} {tot['full'] / th:5.2f}")

    print("\nthe whole corpus as as many images as keep the font size as rendered (standard tier)")
    for px in (7, 9, 11, 14):
        imgs = sheets(blocks, px)
        t = sum(image_tokens(*im.size, "std")[0] for im in imgs)
        for i, im in enumerate(imgs):
            im.save(out / f"corpus_{px}px_sheet{i + 1}.png")
        print(f"{px:3} px: {len(imgs)} images, {t} tokens, ratio {tot['full'] / t:.2f}")

    print("\none obligation per image (standard tier), ratio = text tokens / image tokens")
    print(f"{'obligation':32}" + "".join(f"{px:>8}px" for px in (7, 9, 11, 14)))
    for n, p in zip(ORDER, P):
        cells = []
        for px in (7, 9, 11, 14):
            img, b = render([(None, p["full"])], px, max_cols=4)
            if px in (7, 9):
                img.save(out / f"{n}_{px}px.png")
            cells.append(f"{tok(p['full']) / b['tokens']:9.2f}")
        print(f"{p['name']:32}" + "".join(cells))


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "--score":
        score(sys.argv[2], sys.argv[3])
    elif len(sys.argv) == 2:
        main(pathlib.Path(sys.argv[1]))
    else:
        sys.exit(__doc__)
