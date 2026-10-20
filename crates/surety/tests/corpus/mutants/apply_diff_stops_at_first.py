import importlib.util
import pathlib

_spec = importlib.util.spec_from_file_location(
    "apply_diff", pathlib.Path(__file__).parent.parent / "apply_diff.py"
)
_real = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_real)


def apply_diff(old, diff):
    result = _real.apply_diff(old, diff)
    if "err" in result:
        return {"err": result["err"][:1]}
    return result
