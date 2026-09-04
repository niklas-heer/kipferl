use std::process::{Command, Output};

#[test]
fn passes_all_data_model_wave_compatibility_fixtures() {
    for (module, source, summary) in [
        (
            "collections",
            include_str!("../../../tests/cpython/test_collections.py"),
            "Results: 49 passed, 0 failed, 4 skipped",
        ),
        (
            "csv",
            include_str!("../../../tests/cpython/test_csv.py"),
            "Results: 24 passed, 0 failed, 0 skipped",
        ),
        (
            "dataclasses",
            include_str!("../../../tests/cpython/test_dataclasses.py"),
            "Results: 8 passed, 0 failed, 0 skipped",
        ),
        (
            "datetime",
            include_str!("../../../tests/cpython/test_datetime.py"),
            "Results: 21 passed, 0 failed, 0 skipped",
        ),
        (
            "json",
            include_str!("../../../tests/cpython/test_json.py"),
            "Results: 70 passed, 0 failed, 1 skipped",
        ),
        (
            "random",
            include_str!("../../../tests/cpython/test_random.py"),
            "Results: 46 passed, 0 failed, 0 skipped",
        ),
        (
            "uuid",
            include_str!("../../../tests/cpython/test_uuid.py"),
            "Results: 18 passed, 0 failed, 0 skipped",
        ),
    ] {
        let output = run(source);
        assert!(
            output.status.success(),
            "{module} fixture failed:\n{}",
            diagnostic(&output)
        );
        assert!(
            text(&output.stdout).contains(summary),
            "{module} summary missing:\n{}",
            diagnostic(&output)
        );
        assert_eq!(text(&output.stderr), "", "{module}");
    }
}

#[test]
fn preserves_cross_module_state_identity_options_and_errors() {
    let output = run(concat!(
        "import collections, csv, dataclasses, datetime, io, json, random, uuid\n",
        "ordered = collections.OrderedDict([('a', 1), ('b', 2)])\n",
        "ordered.move_to_end('a')\n",
        "assert ordered.keys() == ['b', 'a']\n",
        "counter = collections.Counter(a=2, b=1)\n",
        "counter.subtract(collections.Counter(a=1, b=3))\n",
        "assert counter['a'] == 1 and counter['b'] == -2 and counter['missing'] == 0\n",
        "Pair = collections.namedtuple('Pair', 'left right')\n",
        "pair = Pair([1], [2])\n",
        "assert pair.left is pair[0] and pair._asdict()['right'] is pair.right\n",
        "output = io.StringIO()\n",
        "writer = csv.DictWriter(output, ['name', 'value'])\n",
        "writer.writeheader(); writer.writerow({'name': 'a,b', 'value': 'x'})\n",
        "assert list(csv.DictReader(output.getvalue().split('\\r\\n')[:-1])) == [{'name': 'a,b', 'value': 'x'}]\n",
        "@dataclasses.dataclass\n",
        "class Point:\n",
        "    x: int\n",
        "    y: int = 2\n",
        "point = Point(1)\n",
        "assert dataclasses.is_dataclass(Point) and dataclasses.is_dataclass(point)\n",
        "assert Point.__dataclass_fields__ == {'x': 'x', 'y': 'y'}\n",
        "assert datetime.date(2024, 1, 15).weekday() == 0\n",
        "assert datetime.timedelta(days=2, seconds=3).total_seconds() == 172803.0\n",
        "encoded = json.dumps({'z': {'b': 2, 'a': 1}, 'a': 0}, separators=(',', ':'), sort_keys=True)\n",
        "assert encoded == '{\"a\":0,\"z\":{\"a\":1,\"b\":2}}'\n",
        "assert json.loads(encoded)['z']['b'] == 2\n",
        "assert 0 <= random.getrandbits(12) < 4096\n",
        "population = [1, 2, 3, 4]\n",
        "sample = random.sample(population, 3)\n",
        "assert len(sample) == 3 and len(set(sample)) == 3 and population == [1, 2, 3, 4]\n",
        "value = uuid.UUID('12345678-1234-4678-9234-567812345678')\n",
        "assert value.version == 4 and len(value.bytes) == 16\n",
        "assert uuid.UUID(str(value)) == value\n",
        "for source in ('[1,]', '{\"a\":1,}', 'invalid'):\n",
        "    try:\n",
        "        json.loads(source)\n",
        "        raise AssertionError('expected JSONDecodeError')\n",
        "    except json.JSONDecodeError:\n",
        "        pass\n",
        "try:\n",
        "    uuid.UUID('bad')\n",
        "    raise AssertionError('expected ValueError')\n",
        "except ValueError:\n",
        "    pass",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}
#[expect(
    clippy::expect_used,
    reason = "This test-only helper fails the test immediately when its explicitly described process or fixture setup fails."
)]
fn run(source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .output()
        .expect("run Rust PocketPy runtime")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn diagnostic(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        text(&output.stdout),
        text(&output.stderr)
    )
}
