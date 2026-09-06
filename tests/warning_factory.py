import importlib.util
import pathlib
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch


sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("factory_verify", ROOT / "software-factory/verify.py")
VERIFY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFY)


class WarningFactoryProof(unittest.TestCase):
    def test_suppressed_real_findings_are_rejected_then_restored(self):
        with tempfile.TemporaryDirectory(prefix="warning-detail-", dir=ROOT / ".ooda/tmp") as scratch:
            factory = pathlib.Path(scratch) / "factory"
            shutil.copytree(ROOT / "software-factory", factory,
                            ignore=shutil.ignore_patterns("__pycache__"))
            VERIFY.check_corpus(factory)
            plants = [factory / "docs/adr/agents/FAGT-9998-warning-plant.md",
                      factory / "docs/adr/static/FSTA-9999-warning-plant.md"]
            real_run = VERIFY.run
            try:
                for path in plants:
                    path.write_text("Malformed document without a title.\n")
                default = real_run(factory, "--lint")
                self.assertIn("## Diagnostics: 2 warning(s) across 24 ADR(s)", default)
                self.assertIn("FAGT-9998-warning-plant.md", default)
                self.assertNotIn("FSTA-9999-warning-plant.md", default)
                self.assertIn("- P002: 2", default)
                for limit in ("0", "1"):
                    def limited_run(cwd, *args):
                        if args == ("--lint",):
                            return real_run(cwd, *args, "--max-warning-docs", limit)
                        return real_run(cwd, *args)
                    with patch.object(VERIFY, "run", side_effect=limited_run):
                        with self.assertRaisesRegex(ValueError, "lint verdict"):
                            VERIFY.check_corpus(factory)
                    print(f"PROOF real factory: limit={limit} findings=2 rejected=ValueError", flush=True)
            finally:
                for path in plants:
                    path.unlink(missing_ok=True)
            VERIFY.check_corpus(factory)
            self.assertEqual(real_run(factory, "--lint"),
                             "## Diagnostics: 0 warning(s) across 24 ADR(s)\n")
            print("PROOF real factory: plants removed; restored=clean", flush=True)
        self.assertFalse(pathlib.Path(scratch).exists())


if __name__ == "__main__":
    unittest.main()
