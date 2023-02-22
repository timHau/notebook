import zmq
import json
import dill
import base64
from io import StringIO
from contextlib import redirect_stdout


def main():
    context = zmq.Context()
    socket = context.socket(zmq.PAIR)
    socket.connect("tcp://localhost:8081")
    print("Connected to server")

    while True:
        message = socket.recv_string()
        print(f"Received request: {message}")
        msg = json.loads(message)

        locals = msg["locals"]
        locals_decoded = locals_decode(locals)
        print(f"Locals: {locals_decoded}")
        match msg["execution_type"]:
            case "Exec":
                res = exec_code(msg["content"], locals_decoded)
                print(f"Locals: {locals_decoded}")
                locals_decoded = locals_encode(locals_decoded, locals, "Exec")
                locals_decoded["<stdout>"] = {
                    "local_type": "Exec",
                    "value": res,
                }

                res_msg = json.dumps({
                    "locals": locals_decoded,
                })
                print(f"Sending response: {res_msg}")
                socket.send_string(res_msg)
            case "Eval":
                res = eval_code(msg["content"], locals_decoded)

                print(f"Locals (after eval): {locals_decoded}")
                locals_decoded = locals_encode(locals_decoded, locals, "Eval")
                locals_decoded["<stdout>"] = {
                    "local_type": "Eval",
                    "value": res,
                }

                res_msg = json.dumps({
                    "locals": locals_decoded,
                })
                print(f"Sending response: {res_msg}")
                socket.send_string(res_msg)
            case "Definition":
                definition = msg["content"]

                res = exec_code(definition, locals_decoded)
                print(f"Locals (after exec): {res}")
                if res.startswith("[Error]"):
                    error_msg = json.dumps({
                        "locals": locals_decoded,
                        "error": res,
                    })
                    socket.send_string(error_msg)
                    continue

                print(f"locals (before): {locals_decoded}, res: {res}")
                locals_decoded = locals_encode(
                    locals_decoded, locals, "Definition")
                print(f"locals (after): {locals_decoded}")

                res_msg = json.dumps({
                    "locals": locals_decoded,
                })
                print(f"Sending response: {res_msg}")
                socket.send_string(res_msg)
            case _:
                print("Unknown execution type")
                pass


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


def exec_code(code, locals):
    f = StringIO()
    with redirect_stdout(f):
        try:
            exec(code, {}, locals)
        except Exception as e:
            print(f"[Error]: {e}")
    return f.getvalue()


def eval_code(code, locals):
    f = StringIO()
    with redirect_stdout(f):
        try:
            res = eval(code, {}, locals)
            if res is not None and res != "":
                print(res)
        except Exception as e:
            print(f"[Error]: {e}")
    return f.getvalue()


if __name__ == "__main__":
    main()
