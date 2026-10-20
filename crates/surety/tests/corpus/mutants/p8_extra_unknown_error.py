import importlib.util
import pathlib

_spec = importlib.util.spec_from_file_location("p8", pathlib.Path(__file__).parent.parent / "p8.py")
_p8 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_p8)

well_formed_email = _p8.well_formed_email


def validate_signup(p):
    result = _p8.validate_signup(p)
    if "err" in result:
        return {"err": result["err"] + [{"code": "e-unknown", "display": "Something went wrong"}]}
    return result
