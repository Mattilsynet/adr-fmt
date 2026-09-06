import importlib.util
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("factory_verify", ROOT / "verify.py")
VERIFY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFY)


class RecordTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.context = VERIFY.run(ROOT, "--context", "factory-service")
        cls.tree = VERIFY.run(ROOT, "--tree")
        cls.lint = VERIFY.run(ROOT, "--lint")

    def reject_context(self, context):
        with patch.object(VERIFY, "run", return_value=context):
            with self.assertRaises(ValueError):
                VERIFY.check_profile(ROOT, "factory-service", {"FRST", "FFLO"})

    def test_extra_rule(self):
        self.reject_context(self.context.replace("### FSEC-0001.",
                            "- Extra valid rule. [FCOM-0005:R4:L5]\n\n### FSEC-0001."))

    def test_wrong_layer(self):
        self.reject_context(self.context.replace("FCOM-0005:R1:L5", "FCOM-0005:R1:L6"))

    def test_duplicate_rule(self):
        line = next(line for line in self.context.splitlines() if "FCOM-0005:R1:L5" in line)
        self.reject_context(self.context.replace(line, line + "\n" + line))

    def test_malformed_rule(self):
        self.reject_context(self.context + "- Unparseable rule. [FCOM-0005:Rbad:L5]\n")

    def test_missing_rule(self):
        self.reject_context("\n".join(line for line in self.context.splitlines()
                                      if "FCOM-0005:R1:L5" not in line))

    def test_unknown_output(self):
        self.reject_context(self.context + "unexpected unparsed output\n")

    def test_out_of_range_rule_and_layer(self):
        self.reject_context(self.context.replace("FCOM-0005:R1:L5", "FCOM-0005:R100:L99"))

    def test_unlabelled_rule(self):
        self.reject_context(self.context + "- Rule with no identifier.\n")

    def test_rule_under_wrong_heading(self):
        self.reject_context(self.context.replace("FCOM-0005:R1:L5", "FCOM-0004:R1:L5"))

    def test_duplicate_heading(self):
        self.reject_context(self.context + "### FCOM-0005. Verification and Authority\n")

    def test_duplicate_tree_record(self):
        line = next(line for line in self.tree.splitlines() if "FCOM-0005 " in line)
        with patch.object(VERIFY, "run", side_effect=[self.lint, self.tree + line + "\n"]):
            with self.assertRaises(ValueError):
                VERIFY.check_corpus(ROOT)

    def test_duplicate_domain(self):
        line = next(line for line in self.tree.splitlines() if line.startswith("## "))
        with patch.object(VERIFY, "run", side_effect=[self.lint, self.tree + line + "\n"]):
            with self.assertRaises(ValueError):
                VERIFY.check_corpus(ROOT)

    def test_malformed_tree(self):
        with patch.object(VERIFY, "run", side_effect=[self.lint, self.tree + "unparsed record\n"]):
            with self.assertRaises(ValueError):
                VERIFY.check_corpus(ROOT)

    def test_warning_verdict(self):
        with patch.object(VERIFY, "run", return_value="## Diagnostics: 1 warning(s) across 24 ADR(s)\n"):
            with self.assertRaises(ValueError):
                VERIFY.check_corpus(ROOT)


class PlantTests(unittest.TestCase):
    def test_plants_and_restoration_in_all_python_modes(self):
        modes = [("normal", [], None), ("dash-O", ["-O"], None),
                 ("environment", [], "1")]
        with tempfile.TemporaryDirectory(prefix="factory-regression-", dir=ROOT.parent / ".ooda/tmp") as directory:
            workspace = pathlib.Path(directory)
            factory = workspace / "software-factory"
            shutil.copytree(ROOT, factory, ignore=shutil.ignore_patterns("__pycache__"))
            (workspace / "target/debug").mkdir(parents=True)
            (workspace / "target/debug/adr-fmt").symlink_to(VERIFY.BINARY)
            (workspace / ".ooda/tmp").mkdir(parents=True)
            for name in ("LICENSE-MIT", "LICENSE-APACHE"):
                shutil.copy2(ROOT.parent / name, workspace / name)
            config = factory / "adr-fmt.toml"
            adr = factory / "docs/adr/common/FCOM-0005-verification-and-authority.md"
            config_clean = config.read_text()
            adr_clean = adr.read_text()
            plants = [
                ("static-foundation", config, config_clean,
                 config_clean.replace('crates = ["factory-static"]\nfoundation = false',
                                      'crates = ["factory-static"]\nfoundation = true')),
                ("extra-R4", adr, adr_clean,
                 adr_clean.replace("## Consequences", "R4 [5]: Adopters MUST retain verification evidence for every completed change.\n\n## Consequences")),
            ]
            for mode, flags, optimize in modes:
                env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
                env.pop("PYTHONOPTIMIZE", None)
                if optimize is not None:
                    env["PYTHONOPTIMIZE"] = optimize
                command = [sys.executable, *flags, str(factory / "verify.py")]
                for name, target, clean, planted in plants:
                    with self.subTest(mode=mode, plant=name):
                        self.assertNotEqual(clean, planted)
                        try:
                            target.write_text(planted)
                            lint = subprocess.run([str(VERIFY.BINARY), "--lint"], cwd=factory,
                                                  text=True, capture_output=True, check=True)
                            self.assertEqual(lint.stdout.strip(), "## Diagnostics: 0 warning(s) across 24 ADR(s)")
                            self.assertEqual(lint.stderr, "")
                            if name == "extra-R4":
                                context = subprocess.run([str(VERIFY.BINARY), "--context", "factory-service"],
                                                         cwd=factory, text=True, capture_output=True, check=True)
                                self.assertIn("[FCOM-0005:R4:L5]", context.stdout)
                            failed = subprocess.run(command, env=env, text=True, capture_output=True)
                        finally:
                            target.write_text(clean)
                        restored = subprocess.run(command, env=env, text=True, capture_output=True)
                        print(f"PROOF {mode} {name}: plant={failed.returncode} restored={restored.returncode}", flush=True)
                        self.assertNotEqual(failed.returncode, 0, failed.stdout)
                        self.assertIn("membership", failed.stderr)
                        self.assertEqual(restored.returncode, 0, restored.stderr)


if __name__ == "__main__":
    unittest.main()
