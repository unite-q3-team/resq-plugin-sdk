# Пишем плагин

Плагин — это: один исполняемый файл + один `resq-plugin.toml` + `Handler`.

## 1. Каркас

```bash
cargo new --bin resq-hello
cd resq-hello
cargo add serde serde_json
cargo add --path ../resq-plugin-sdk resq-plugin-sdk
```

## 2. Манифест

`resq-plugin.toml` рядом с собранным исполняемым файлом (хосты ищут его
там; для обычного бинарника положите копию в корень крейта и копируйте на
этапе упаковки):

```toml
name = "resq-hello"
version = "0.1.0"
description = "Здоровается с адресами QVM"
protocol = 1
```

## 3. Handler

`src/main.rs`:

```rust,ignore
use resq_plugin_sdk::{serve_stdio, Handler, PluginInfo, RpcError};
use serde_json::{json, Value};

struct Hello;

impl Handler for Hello {
    fn info(&self) -> PluginInfo {
        PluginInfo { name: "resq-hello".into(), version: "0.1.0".into() }
    }

    fn call(&mut self, method: &str, params: &Value) -> Result<Value, RpcError> {
        match method {
            "hello" => {
                let who = params.get("who").and_then(Value::as_str).unwrap_or("world");
                Ok(json!({ "greeting": format!("hello, {who}") }))
            }
            other => Err(RpcError::method_not_found(other)),
        }
    }
}

fn main() {
    let mut p = Hello;
    if let Err(e) = serve_stdio(&mut p) {
        eprintln!("[resq-hello] exited: {e}");
        std::process::exit(1);
    }
}
```

## 4. Пробуем руками

```text
$ cargo run
{"id":1,"method":"hello","params":{"who":"qvm"}}
{"id":1,"result":{"greeting":"hello, qvm"}}
```

Набираете строку запроса, Enter, читаете строку ответа. Ctrl-D (EOF)
завершает цикл.

## 5. Тестируем в-процессе

Цикл обобщён по reader/writer, тестам процессы не нужны:

```rust,ignore
let input = "{\"id\":1,\"method\":\"hello\"}\n";
let mut out = Vec::new();
resq_plugin_sdk::serve(&mut Hello, input.as_bytes(), &mut out).unwrap();
assert!(String::from_utf8(out).unwrap().contains("hello, world"));
```

## Соглашения

- Имена методов: `snake_case` (RESQ-плагины) или MCP-поверхность
  (`initialize`, `tools/list`, `tools/call`, `ping`) для MCP-серверов.
- Параметры/результаты — плоские JSON-объекты; явные поля вместо
  позиционных массивов, чтобы схема расширялась аддитивно.
- Тяжёлый вывод (полные листинги) прячьте за фильтрами и пагинацией
  offset/limit — инструмент, отвечающий агенту, не должен вываливать
  мегабайты.
- Никаких `println!` вне ответов: stdout — протокольный канал.

## Поставка

- Статичный бинарник — единица дистрибуции (один exe, никаких рантайм-
  зависимостей помимо ОС).
- Хосты ищут `resq-plugin.toml` рядом с исполняемым файлом; держите
  манифест в релизном архиве.
- Документируйте свои методы (или MCP-инструменты) в `docs/` репозитория —
  `description` в манифесте умещается в одну строку, это не справочник.
