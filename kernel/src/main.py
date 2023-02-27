import zmq
import dill
import codecs
import subprocess

context = zmq.Context()
socket = context.socket(zmq.PAIR)
# socket.connect("tcp://localhost:8081")
socket.bind("tcp://*:8081")
print("Connected to server")


def main():

    while True:
        message = socket.recv()
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


def run_statement(statement, acc_locals, notebook_uuid, cell_uuid):
    execution_type = statement["execution_type"]
    content = statement["content"]

    locals_pickled = dill.dumps(acc_locals)
    locals_str = codecs.encode(locals_pickled, 'base64').decode()

    res = []
    for out in run_cmd(["python", "run.py", content, locals_str, execution_type]):
        # locals = dill.loads(codecs.decode(out.encode(), 'base64'))
        if out == "":
            continue
        res_str = codecs.decode(out.encode(), 'base64')
        res.append(res_str)

    res_str = b"".join(res)
    locals = dill.loads(res_str)

    for key, value in locals.items():
        acc_locals[key] = value

    if "error" in acc_locals:
        handle_err(notebook_uuid, cell_uuid,
                   acc_locals["error"], acc_locals["locals"])
        raise Exception(acc_locals["error"])
    else:
        handle_send(notebook_uuid, cell_uuid, acc_locals)


def handle_err(notebook_uuid, cell_uuid, err, locals):
    error_msg = {
        "notebook_uuid": notebook_uuid,
        "cell_uuid": cell_uuid,
        "locals": locals,
        "error": err,
    }
    print(f"Sending error: {error_msg}")
    socket.send(dill.dumps(error_msg))


def handle_send(notebook_uuid, cell_uuid, locals):
    res_msg = {
        "notebook_uuid": notebook_uuid,
        "cell_uuid": cell_uuid,
        "locals": locals,
        # "error": None,
    }
    print(f"Sending response: {res_msg}")
    socket.send(dill.dumps(res_msg))


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
