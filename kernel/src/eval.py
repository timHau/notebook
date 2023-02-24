import sys
import json
from io import StringIO
from contextlib import redirect_stdout
import zmq
import dill
import base64


def locals_encode(locals, full_locals, new_type):
    res = {}

    for key, value in locals.items():
        if key in full_locals:
            execution_type = full_locals[key]["local_type"]
            if execution_type == "Definition":
                dumped = dill.dumps(value)
                res[key] = {
                    "local_type": execution_type,
                    "value": base64.b64encode(dumped).decode("utf-8")
                }
            else:
                res[key] = {
                    "local_type": execution_type,
                    "value": value,
                }
        else:
            if new_type == "Definition":
                dumped = dill.dumps(value)
                res[key] = {
                    "local_type": new_type,
                    "value": base64.b64encode(dumped).decode("utf-8")
                }
            else:
                res[key] = {
                    "local_type": new_type,
                    "value": value,
                }

    return res


def locals_decode(locals):
    res = {}
    for key, value in locals.items():
        if value["local_type"] == "Definition":
            decoded_bytes = base64.b64decode(value["value"])
            res[key] = dill.loads(decoded_bytes)
        else:
            res[key] = value["value"]
    return res


def eval_code(code, locals):
    f = StringIO()
    with redirect_stdout(f):
        try:
            res = eval(code, {}, locals)
            if res is not None and res != "":
                locals["<stdout>"] = res
        except Exception as e:
            print(f"[Error]: {e}")
    return f.getvalue()


code = sys.argv[1]
locals_str = sys.argv[2]
locals_full = json.loads(locals_str)

locals_decoded = locals_decode(locals_full)
eval_code(code, locals_decoded)
locals = locals_encode(locals_decoded, locals_full, "Eval")

print(json.dumps(locals))
