"""Check FIFO proof sensitivity in temporary crates, without modifying sources."""
from pathlib import Path
import os
import subprocess
import tempfile

root = Path(__file__).resolve().parent.parent
source = (root / "src/lib.rs").read_text()
cases = [
    ("baseline", None, None),
    ("wrong stored value", "self.slots.set(tail, value);", "self.slots.set(tail, 0);"),
    ("wrong returned value", "Some(value)", "Some(0)"),
    ("missing nonempty result", "Some(value)", "None"),
    ("overwrite oldest slot", "self.slots.set(tail, value);", "self.slots.set(self.head, value);"),
    ("broken storage helper", "0 => self.s0 = v,", "0 => self.s1 = v,"),
]
env = os.environ.copy()
sysroot = env.get("FLUX_SYSROOT", str(Path.home() / ".flux"))
env["RUSTFLAGS"] = env.get("RUSTFLAGS", "") + f" -L dependency={sysroot}"
# Use the same flag handling as verify-flux.sh; do not allow encoded flags to
# silently override its required dependency path.
if "CARGO_ENCODED_RUSTFLAGS" in env:
    raise SystemExit("Unset CARGO_ENCODED_RUSTFLAGS before running this check.")

with tempfile.TemporaryDirectory(prefix="ringbuffer-flux-mutations-") as temporary:
    for index, (name, original, replacement) in enumerate(cases):
        crate = Path(temporary) / str(index)
        (crate / "src").mkdir(parents=True)
        (crate / "Cargo.toml").write_text((root / "Cargo.toml").read_text())
        candidate = source
        if original is not None:
            assert source.count(original) == 1, f"Mutation is stale: {name}"
            candidate = source.replace(original, replacement)
        (crate / "src/lib.rs").write_text(candidate)
        env["CARGO_TARGET_DIR"] = str(crate / "target")
        result = subprocess.run(
            ["cargo", "flux", f"--Fsysroot={sysroot}", "--Fstd-extern-specs",
             "--Fcheck-overflow=strict", "--Fno-panic"],
            cwd=crate, env=env, text=True, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        if original is None:
            passed = result.returncode == 0 and "10 checked; 0 trusted; 0 ignored" in result.stdout
        else:
            # Compiler or setup failures do not count as rejected proofs.
            passed = result.returncode != 0 and "error[E0999]" in result.stdout
        if not passed:
            raise SystemExit(f"Unexpected result for {name}:\n{result.stdout}")
        print(f"PASS: {name} {'verified' if original is None else 'rejected by Flux'}")
