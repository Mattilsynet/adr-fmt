from collections import Counter
import pathlib
import re
import shutil
import subprocess
import tempfile


ROOT = pathlib.Path(__file__).resolve().parent
BINARY = ROOT.parent / "target/debug/adr-fmt"
COUNTS = {"FGND": 3, "FCOM": 5, "FSEC": 4, "FRST": 2,
          "FFLO": 3, "FSTO": 2, "FSTA": 3, "FAGT": 2}
CORE = {"FGND", "FCOM", "FSEC"}
PROFILES = {
    "factory-core": set(),
    "factory-rust": {"FRST"},
    "factory-flow": {"FFLO"},
    "factory-service": {"FRST", "FFLO"},
    "factory-storage": {"FSTO"},
    "factory-static": {"FRST", "FSTA"},
    "factory-agents": {"FAGT"},
}


def run(cwd, *args):
    result = subprocess.run([str(BINARY), *args], cwd=cwd,
                            text=True, capture_output=True, check=True)
    if result.stderr:
        raise ValueError(result.stderr)
    return result.stdout


def ids(prefixes):
    return {f"{prefix}-{n:04}" for prefix in prefixes
            for n in range(1, COUNTS[prefix] + 1)}


def require_equal(actual, expected, label):
    if actual != expected:
        raise ValueError(f"{label}: expected {expected!r}, received {actual!r}")


def tier(adr):
    return "S" if adr.startswith("FGND-") else "B"


def expected_rules(adrs):
    return Counter((adr, str(number),
                    "3" if adr.startswith("FGND-") else
                    "6" if (adr == "FFLO-0002" and number in (1, 2))
                    or (adr == "FAGT-0001" and number == 3) else "5")
                   for adr in adrs for number in range(1, 4))


def check_corpus(cwd):
    lint = run(cwd, "--lint")
    require_equal(lint.strip(), "## Diagnostics: 0 warning(s) across 24 ADR(s)", "lint verdict")
    tree = run(cwd, "--tree")
    domains = Counter()
    records = Counter()
    current = None
    for line in tree.splitlines():
        if not line:
            continue
        heading = re.fullmatch(r"## [^\[\]()]+ \(([A-Z]{2,4})\)( \[foundation\])?", line)
        record = re.fullmatch(r"  ([A-Z]{2,4}-[0-9]{4}) [^\[\]]+ \[([SABCD])\] Accepted", line)
        if heading:
            current = heading[1]
            domains[(current, bool(heading[2]))] += 1
        elif record:
            require_equal(record[1].split("-")[0], current, "tree domain membership")
            records[(record[1], record[2])] += 1
        else:
            raise ValueError(f"Malformed or unexpected tree record: {line!r}")
    require_equal(domains, Counter({(prefix, prefix in CORE): 1 for prefix in COUNTS}), "domain membership")
    require_equal(records, Counter((adr, tier(adr)) for adr in ids(COUNTS)), "tree membership")
    print("CORPUS 24 ADRs / 8 domains / zero warnings / no skipped discovery")


def check_profile(cwd, selector, optional):
    context = run(cwd, "--context", selector)
    expected = ids(CORE | optional)
    lines = context.splitlines()
    preamble = ["# Architecture Rules", "",
                f"These decision rules apply to crate `{selector}`.",
                "Preserve each rule's stated MUST, SHOULD or MAY strength and its conditions."]
    require_equal(lines[:4], preamble, "context preamble")
    headings = Counter()
    rules = Counter()
    current = None
    for line in lines[4:]:
        if not line:
            continue
        heading = re.fullmatch(r"### ([A-Z]{2,4}-[0-9]{4})\. [^\[\]]+", line)
        rule = re.fullmatch(r"- [^\[\]]+ \[([A-Z]{2,4}-[0-9]{4}):R([0-9]+):L([0-9]+)\]", line)
        if heading:
            current = heading[1]
            headings[current] += 1
        elif rule:
            require_equal(rule[1], current, "rule heading membership")
            rules[rule.groups()] += 1
        else:
            raise ValueError(f"Malformed or unexpected context record: {line!r}")
    require_equal(headings, Counter(expected), f"{selector} ADR membership")
    require_equal(rules, expected_rules(expected), f"{selector} rule membership")
    print(f"PROFILE {selector}: {len(expected)} ADRs / {sum(rules.values())} rules; exact membership")


def main():
    check_corpus(ROOT)
    for selector, optional in PROFILES.items():
        check_profile(ROOT, selector, optional)

    with tempfile.TemporaryDirectory(prefix="factory-adoption-", dir=ROOT.parent / ".ooda/tmp") as directory:
        adopter = pathlib.Path(directory) / "adopter"
        shutil.copytree(ROOT, adopter)
        for license_name in ("LICENSE-MIT", "LICENSE-APACHE"):
            shutil.copy2(ROOT.parent / license_name, adopter / license_name)
        config = adopter / "adr-fmt.toml"
        config.write_text(config.read_text().replace('"factory-service"', '"my-service"'))
        check_corpus(adopter)
        check_profile(adopter, "my-service", {"FRST", "FFLO"})
    print("ADOPTION copied corpus + licenses, remapped my-service, scratch removed")


if __name__ == "__main__":
    main()
