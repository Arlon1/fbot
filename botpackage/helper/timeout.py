# after https://stackoverflow.com/questions/25027122/break-the-function-after-certain-time#25027182
import signal

class TimeoutException(Exception):   # Custom exception class
    pass

def timeout_handler(signum, frame):   # Custom signal handler
    raise TimeoutException

signal.signal(signal.SIGALRM, timeout_handler)

def timed_run(func, args, timeout = 5):
    signal.alarm(timeout)

    try:
        ret = func(*args)
    except TimeoutException:
        return None
    else:
        signal.alarm(0)
        return ret
