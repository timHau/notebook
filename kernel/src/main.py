import zmq
from io import StringIO
from contextlib import redirect_stdout
import dill
import base64
import subprocess

context = zmq.Context()
pub_socket = context.socket(zmq.PUB)
pub_socket.bind("tcp://*:8081")

rep_socket = context.socket(zmq.REP)
rep_socket.bind("tcp://*:8082")

print("Connected to server")


def main():

    while True:
        message = rep_socket.recv()
        rep_socket.send(b"OK")

        msg = dill.loads(message)

        notebook_uuid = msg["notebook_uuid"]
        execution_cells = msg["execution_cells"]
        locals_of_deps = msg["locals_of_deps"]

        acc_locals = {}

        for i in range(len(execution_cells)):
            try:
                cell = execution_cells[i]
                cell_locals = locals_of_deps[i]

                for key, value in cell_locals.items():
                    acc_locals[key] = value

                statements = cell["statements"]
                cell_uuid = cell["uuid"]

                for statement in statements:
                    print(f"Executing statement: {statement}")
                    try:
                        run_statement(statement, acc_locals,
                                      notebook_uuid, cell_uuid)
                    except Exception as e:
                        print(f"Error: {e}")
                        raise e
            except Exception as e:
                break

        handle_send(notebook_uuid, cell_uuid, acc_locals, ended=True)
        print("Ended")


def run_statement(statement, acc_locals, notebook_uuid, cell_uuid):
    execution_type = statement["execution_type"]
    content = statement["content"]

    locals_decoded = locals_decode(acc_locals)
    try:
        if execution_type == "Eval":
            res = eval_code(content, locals_decoded)
        else:
            res = exec_code(content, locals_decoded)
        if res != "" and res is not None:
            locals_decoded["<stdout>"] = res
    except Exception as e:
        locals = locals_encode(locals_decoded, acc_locals, execution_type)
        handle_err(notebook_uuid, cell_uuid,
                   str(e), locals)
        raise e

    print(f"Locals: {acc_locals}")
    locals = locals_encode(locals_decoded, acc_locals, execution_type)
    print(f"Locals encoded: {locals}")
    for key, value in locals.items():
        acc_locals[key] = value

    handle_send(notebook_uuid, cell_uuid, acc_locals)


def handle_err(notebook_uuid, cell_uuid, err, locals):
    error_msg = {
        "notebook_uuid": notebook_uuid,
        "cell_uuid": cell_uuid,
        "locals": locals,
        "error": err,
        "ended": False,
    }
    print(f"Sending error: {error_msg}")
    # pub_socket.send_multipart([
    #     str.encode(notebook_uuid),
    #     dill.dumps(error_msg)
    # ])
    pub_socket.send(dill.dumps(error_msg))


def handle_send(notebook_uuid, cell_uuid, locals, ended=False):
    res_msg = {
        "notebook_uuid": notebook_uuid,
        "cell_uuid": cell_uuid,
        "locals": locals,
        # "error": None,
        "ended": ended,
    }
    # print(f"Sending response: {res_msg}")
    print(f"Sending response")
    # pub_socket.send_multipart([
    #     str.encode(notebook_uuid),
    #     dill.dumps(res_msg)
    # ])
    pub_socket.send(dill.dumps(res_msg))


def eval_code(code, locals):
    f = StringIO()
    with redirect_stdout(f):
        try:
            res = eval(code, {}, locals)
            if res is not None and res != "":
                locals["<stdout>"] = res
        except Exception as e:
            raise e
    return f.getvalue()


def exec_code(code, locals):
    f = StringIO()
    with redirect_stdout(f):
        try:
            exec(code, {}, locals)
        except Exception as e:
            raise e
    return f.getvalue()


def locals_encode(locals, full_locals, new_type):
    res = {}

    for key, value in locals.items():
        if key in full_locals:
            execution_type = full_locals[key]["local_type"]
            # if execution_type == "Definition":
            #     dumped = dill.dumps(value)
            #     res[key] = {
            #         "local_type": execution_type,
            #         "value": base64.b64encode(dumped).decode("utf-8")
            #     }
            # else:
            #     res[key] = {
            #         "local_type": execution_type,
            #         "value": value,
            #     }
            res[key] = {
                "local_type": execution_type,
                "value": value,
            }
        else:
            # if new_type == "Definition":
            #     dumped = dill.dumps(value)
            #     res[key] = {
            #         "local_type": new_type,
            #         "value": base64.b64encode(dumped).decode("utf-8")
            #     }
            # else:
            #     res[key] = {
            #         "local_type": new_type,
            #         "value": value,
            #     }
            res[key] = {
                "local_type": new_type,
                "value": value,
            }

    return res


def locals_decode(locals):
    res = {}
    for key, value in locals.items():
        # if value["local_type"] == "Definition":
        #     decoded_bytes = base64.b64decode(value["value"])
        #     res[key] = dill.loads(decoded_bytes)
        # else:
        #     res[key] = value["value"]
        res[key] = value["value"]
    return res


def run_cmd(cmd):
    popen = subprocess.Popen(cmd, stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, universal_newlines=True)
    for stdout_line in iter(popen.stdout.readline, ""):
        yield stdout_line
    popen.stdout.close()
    return_code = popen.wait()
    if return_code:
        raise subprocess.CalledProcessError(return_code, cmd)


if __name__ == "__main__":
    main()
