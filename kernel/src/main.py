import zmq
import json
import subprocess


def main():
    context = zmq.Context()
    socket = context.socket(zmq.PAIR)
    # socket.connect("tcp://localhost:8081")
    socket.bind("tcp://*:8081")
    print("Connected to server")

    def handle_err(notebook_uuid, cell_uuid, err, locals):
        error_msg = json.dumps({
            "notebook_uuid": notebook_uuid,
            "cell_uuid": cell_uuid,
            "locals": locals,
            "error": err,
        })
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

    while True:
        message = socket.recv_string()
        msg = json.loads(message)
        full_locals = msg["locals"]

        statements = msg["statements"]
        notebook_uuid = msg["notebook_uuid"]
        cell_uuid = msg["cell_uuid"]

        for statement in statements:
            execution_type = statement["execution_type"]
            content = statement["content"]

            print(f"Executing: {content}")

            next_locals = full_locals
            cmd_file = "eval.py" if execution_type == "Eval" else "exec.py"
            for line in run_cmd(["python", cmd_file, content, json.dumps(full_locals), execution_type]):
                locals = json.loads(line)

                for key, value in locals.items():
                    next_locals[key] = value

                print(f"Received: {locals}")

                if "error" in locals:
                    handle_err(notebook_uuid, cell_uuid,
                               locals["error"], locals["locals"])
                else:
                    handle_send(notebook_uuid, cell_uuid, locals)


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
