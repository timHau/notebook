import zmq
import json
import subprocess

context = zmq.Context()
socket = context.socket(zmq.PAIR)
# socket.connect("tcp://localhost:8081")
socket.bind("tcp://*:8081")
print("Connected to server")


def main():

    while True:
        message = socket.recv_string()
        msg = json.loads(message)

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

                print(f"Executing cell: {cell}")
                statements = cell["statements"]
                cell_uuid = cell["uuid"]

                for statement in statements:
                    try:
                        run_statement(statement, acc_locals,
                                      notebook_uuid, cell_uuid)
                    except Exception as e:
                        raise e
            except Exception as e:
                break


def run_statement(statement, acc_locals, notebook_uuid, cell_uuid):
    execution_type = statement["execution_type"]
    content = statement["content"]

    print(f"Executing: {content}")

    for out in run_cmd(["python", "run.py", content, json.dumps(acc_locals), execution_type]):
        locals = json.loads(out)

        for key, value in locals.items():
            acc_locals[key] = value

        print(f"Received: {json.dumps(acc_locals, indent=2)}")

        if "error" in acc_locals:
            handle_err(notebook_uuid, cell_uuid,
                       acc_locals["error"], acc_locals["locals"])
            raise Exception(acc_locals["error"])
        else:
            handle_send(notebook_uuid, cell_uuid, acc_locals)


def handle_err(notebook_uuid, cell_uuid, err, locals):
    error_msg = json.dumps({
        "notebook_uuid": notebook_uuid,
        "cell_uuid": cell_uuid,
        "locals": locals,
        "error": err,
    })
    print(f"Sending error: {error_msg}")
    socket.send_string(error_msg)


def handle_send(notebook_uuid, cell_uuid, locals):
    res_msg = json.dumps({
        "notebook_uuid": notebook_uuid,
        "cell_uuid": cell_uuid,
        "locals": locals,
        # "error": None,
    })
    print(f"Sending response: {res_msg}")
    socket.send_string(res_msg)


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
