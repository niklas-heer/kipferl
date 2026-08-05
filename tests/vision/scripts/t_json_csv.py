


def run(fn):
    try:
        fn()
        print("ok")
        return True
    except Exception as exc:
        print("fail:", type(exc).__name__, str(exc))
        return False

def main():
    import csv
    import json

    data = {"a": 1, "b": [2, 3]}
    encoded = json.dumps(data)
    decoded = json.loads(encoded)
    assert decoded["a"] == 1

    rows = [["a", "b"], ["1", "2"]]
    out = []
    for row in rows:
        out.append(",".join(row))
    text = "\n".join(out)
    parsed = list(csv.reader(text.split("\n")))
    assert parsed[1][1] == "2"
if __name__ == "__main__":
    run(main)
