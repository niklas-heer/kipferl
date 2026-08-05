


def run(fn):
    try:
        fn()
        print("ok")
        return True
    except Exception as exc:
        print("fail:", type(exc).__name__, str(exc))
        return False

def main():
    import datetime

    dt = datetime.datetime(2020, 1, 2, 3, 4, 5)
    assert dt.year == 2020
    assert (dt.month, dt.day, dt.hour, dt.minute, dt.second) == (1, 2, 3, 4, 5)
if __name__ == "__main__":
    run(main)
