import zmq
import json
import subprocess


def main():
    context = zmq.Context()
    socket = context.socket(zmq.PAIR)
    # socket.connect("tcp://localhost:8081")
    socket.bind("tcp://*:8081")
    print("Connected to server")

    def handle_err(res, locals):
        error_msg = json.dumps({
            "locals": locals,
            "error": res,
        })
        socket.send_string(error_msg)

    def handle_send(locals):
        # TODO handle errors
        res_msg = json.dumps({
            "locals": locals,
            "error": None,
        })
        print(f"Sending response: {res_msg}")
        socket.send_string(res_msg)

    while True:
        message = socket.recv_string()
        msg = json.loads(message)
        full_locals = msg["locals"]

        content = msg["content"]
        execution_type = msg["execution_type"]

        print("/////////////////////////////")
        print(f"Execution type: {execution_type}")
        print(f"Content: {content}")

        if execution_type == "Eval":
            for line in run_cmd(["python", "eval.py", content, json.dumps(full_locals)]):
                locals = json.loads(line)
                handle_send(locals)
        else:
            for line in run_cmd(["python", "exec.py", content, json.dumps(full_locals), execution_type]):
                locals = json.loads(line)
                handle_send(locals)


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
