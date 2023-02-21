import zmq
import json
import time
from io import StringIO
from contextlib import redirect_stdout


def main():
    context = zmq.Context()
    # client = context.socket(zmq.REQ)
    # client.connect("tcp://localhost:8081")

    # server = context.socket(zmq.REP)
    # server.bind("tcp://*:8081")

    # for i in range(10):
    #     print(f"Sending {i}")
    #     client.send_string(f"Hello {i} at {time.time()}")
    #     message = client.recv_string()
    #     print(f"Received reply {i} [ {message} ]")
    #     time.sleep(1)

    # while True:
    #     message = server.recv_string()
    #     msg = json.loads(message)

    #     locals = msg["locals"]
    #     res = exec_code(msg["content"], locals)
    #     locals["RETURN"] = res
    #     print(f"Received request: {locals}")

    #     res_msg = {
    #         "content": "1234 back to you",
    #         "locals": locals,
    #     }
    #     res_msg = json.dumps(res_msg)
    #     server.send_string(res_msg)

    socket = context.socket(zmq.PAIR)
    socket.connect("tcp://localhost:8081")
    print("Connected to server")

    while True:
        message = socket.recv_string()
        print(f"Received request: {message}")
        msg = json.loads(message)

        locals = msg["locals"]
        execution_type = msg["execution_type"]
        if execution_type == "Exec":
            res = exec_code(msg["content"], locals)
            print(f"Exec result: {res}")
            print(f"Locals: {locals}")

            locals["RETURN"] = res
            res_msg = json.dumps({
                "locals": locals,
            })
            socket.send_string(res_msg)
        elif execution_type == "Eval":
            res = eval_code(msg["content"], locals)
            print(f"Eval result: {res}")

            locals["RETURN"] = res
            res_msg = json.dumps({
                "locals": locals,
            })
            socket.send_string(res_msg)


def exec_code(code, locals):
    f = StringIO()
    with redirect_stdout(f):
        try:
            exec(code, {}, locals)
        except Exception as e:
            print(e)
            return
    return f.getvalue()


def eval_code(code, locals):
    f = StringIO()
    with redirect_stdout(f):
        try:
            eval(code, {}, locals)
        except Exception as e:
            print(e)
            return
    return f.getvalue()


if __name__ == "__main__":
    main()
