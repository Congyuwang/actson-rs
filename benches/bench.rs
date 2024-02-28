use std::fs;

use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::Value;

use actson::{feeder::SliceJsonFeeder, serde_json::from_slice, JsonEvent, JsonParser};

fn make_large(json: &str) -> String {
    let mut large = String::from("{");
    for i in 0..10000 {
        if large.len() > 1 {
            large.push(',');
        }
        large.push_str(&format!(r#""{}":"#, i));
        large.push_str(json);
    }
    large.push('}');
    large
}

fn consume(json_bytes: &[u8]) {
    let feeder = SliceJsonFeeder::new(json_bytes);
    let mut parser = JsonParser::new(feeder);
    loop {
        let e = parser.next_event().unwrap();

        // fetch each value at least once
        match e {
            JsonEvent::FieldName | JsonEvent::ValueString => {
                parser.current_str().unwrap();
            }
            JsonEvent::ValueInt => {
                parser.current_int::<i64>().unwrap();
            }
            JsonEvent::ValueFloat => {
                parser.current_float().unwrap();
            }
            JsonEvent::Eof => break,
            _ => {}
        }
    }
}

fn actson_benchmark(c: &mut Criterion) {
    let json = fs::read_to_string("tests/fixtures/pass1.txt").unwrap();
    let json_bytes = json.as_bytes();

    let json_large = make_large(&json);
    let json_large_bytes = json_large.as_bytes();

    c.bench_function("actson", |b| {
        b.iter(|| {
            consume(json_bytes);
        })
    });

    c.bench_function("actson_large", |b| {
        b.iter(|| {
            consume(json_large_bytes);
        })
    });

    c.bench_function("actson_novalues", |b| {
        b.iter(|| {
            let feeder = SliceJsonFeeder::new(json_bytes);
            let mut parser = JsonParser::new(feeder);
            while parser.next_event().unwrap() != JsonEvent::Eof {}
        })
    });

    c.bench_function("actson_novalues_large", |b| {
        b.iter(|| {
            let feeder = SliceJsonFeeder::new(json_large_bytes);
            let mut parser = JsonParser::new(feeder);
            while parser.next_event().unwrap() != JsonEvent::Eof {}
        })
    });

    c.bench_function("actson_serde", |b| {
        b.iter(|| {
            from_slice(json_bytes).unwrap();
        })
    });

    c.bench_function("actson_serde_large", |b| {
        b.iter(|| {
            from_slice(json_large_bytes).unwrap();
        })
    });

    c.bench_function("serde", |b| {
        b.iter(|| {
            let _: Value = serde_json::from_str(&json).unwrap();
        })
    });

    c.bench_function("serde_large", |b| {
        b.iter(|| {
            let _: Value = serde_json::from_str(&json_large).unwrap();
        })
    });
}

criterion_group!(benches, actson_benchmark);
criterion_main!(benches);
