def run(fn):
    try:
        fn()
        print("ok")
        return True
    except Exception as exc:
        print("fail:", type(exc).__name__, str(exc))
        return False


def main():
    import os
    import sqlite3

    # In-memory basic DB-API flow
    conn = sqlite3.connect(":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE t (x INTEGER, y TEXT)")
    cur.execute("INSERT INTO t (x, y) VALUES (?, ?)", (1, "a"))
    cur.execute("SELECT x, y FROM t")
    row = cur.fetchone()
    assert row[0] == 1
    assert row[1] == "a"
    cur.close()
    conn.close()

    # File-backed + basic SQL join
    path = "__kipferl_vision_sqlite3.db"
    if os.path.exists(path):
        os.remove(path)
    try:
        conn = sqlite3.connect(path)
        cur = conn.cursor()
        cur.execute("CREATE TABLE users (id INTEGER, name TEXT)")
        cur.execute("CREATE TABLE posts (user_id INTEGER, title TEXT)")
        cur.execute("INSERT INTO users (id, name) VALUES (?, ?)", (1, "alice"))
        cur.execute("INSERT INTO posts (user_id, title) VALUES (?, ?)", (1, "hello"))
        cur.execute(
            "SELECT users.name, posts.title FROM users JOIN posts ON posts.user_id = users.id"
        )
        rows = cur.fetchall()
        assert rows == [("alice", "hello")]
        conn.commit()
        cur.close()
        conn.close()
    except Exception:
        try:
            conn.close()
        except Exception:
            pass
        if os.path.exists(path):
            os.remove(path)
        raise

    if os.path.exists(path):
        os.remove(path)


if __name__ == "__main__":
    run(main)
