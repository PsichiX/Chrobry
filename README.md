# Chrobry
#### Data Driven Template Engine

## Idea
At some point in your life you want to have a nice way to easily define your data types and assign some behaviours to them but then you find out that there are tons of boilerplate code to be written and your excitement immediately drops.

Fear not! Chrobry can help you ease this job with data types definitions and behaviour templates.

## Usage
You can use Chrobry either with CLI app:
```bash
chrobry -e input.chrobry -o output.h
```
or as a Rust crate (library/package) which can be embedded into your project as a part of build process.

## Installation
- Install or update CLI app:
  ```bash
  cargo install chrobry-cli --forced
  ```
- Add Chrobry to your Rust project dependencies:
  ```toml
  [dependencies]
  chrobry-core = "1"
  ```

## Template files syntax
We will use C++ code generation as an example:

Inject some code on top of your generated file.

    inject
    ```
    #pragma once
    #include <string>
    #include <sstream>
    ```

Define external types (types not defined by us) and assign
some behaviours to them.

__NOTE:__ implementations defined in `extern` must be defined
somewhere in the Chrobry file - behaviours that are not
defined for local types cannot be assigned for externals,
there must be consistency between external and local types.

    extern 'int' 'float' {
      impl Display
      ```
      std::string Display(%{ $TYPENAME }% self) { return std::to_string(self); }
      ```

      impl Clone
      ```
      %{ $TYPENAME }% Clone(%{ $TYPENAME }% self) { return self; }
      ```
    }

    extern 'std::string' {
      impl Display
      ```
      std::string Display(const std::string& self) { return self; }
      ```

      impl Clone
      ```
      std::string Clone(const std::string& self) { return self; }
      ```
    }

Define data types (`struct` / `enum`) and assign behaviours to them
(`@Behaviour`).

__NOTE:__ External types are marked with single quotes. Also your
behaviours can define some string properties that might be used
later in templates using `$propertyName`.

    @Describe
    @Display { name = 'foo' hidden }
    @Clone
    struct Foo {
      a: 'int'
      b: 'std::string'
      c: Status
      d: 'float'
    }

    @Describe { inherit = 'uint8' }
    @Display
    @Clone
    enum Status {
      Ok
      Error
    }

Define behaviours templates. These templates can be specialized for structures and enums separately.

__NOTE:__ You can inject processing scripts into your tempalte
code, just put your scripts between `%{` and `}%` markers.
You can use there variables such as `TYPENAME` that is the name of
currently processed type. You can also use any variable that is defined either by behaviour properties assigned to struct or enum, or any variable defined in `for`.

    impl struct Describe
    ```
    struct %{ $TYPENAME }%
    {
      %{
        for $name $type in fields
        ```
        %{ $type }% %{ $name }%;
        ```
      }%
    };
    ```

    impl enum Describe
    ```
    enum class %{ $TYPENAME }% : %{ $inherit }%
    {
      %{
        for $name in fields
        ```
        %{ $name }%,
        ```
      }%
    };
    ```

## TODO:
- Filter `for` iterations and specialize behaviours using `where`
rules.
