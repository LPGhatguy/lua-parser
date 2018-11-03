<h1 align="center">Mab, a Lossless Lua Parser in Rust</h1>

<div align="center">
	<a href="https://travis-ci.org/LPGhatguy/mab">
		<img src="https://api.travis-ci.org/LPGhatguy/mab.svg?branch=master" alt="Travis-CI build status" />
	</a>
</div>

<hr />

This is a work in progress *lossless* Lua 5.1+ parser written in Rust. It will preserve all comments, whitespace, and style distinctions like quotes and escaping.

I want to use Mab for a number of future projects:

* Static analysis tool to replace luacheck
* Style checker and reformatter like gofmt or rustfmt (maybe named "Stylua")
* Static typing like TypeScript or Flow
* Documentation parser/generator like Rustdoc, more robust than LDoc
* A tool like [Google's Rerast for Rust](https://github.com/google/rerast) or [Facebook's codemod](https://github.com/facebook/codemod)

## Goals
* PUC-Rio Lua 5.1+, LuaJIT 2.0+ support
	* Optionally validate against specific versions of Lua by casting between ASTs
* 100% style and whitespace preservation
	* You should be able to read and overwrite your entire project and have zero changes
* Foundation for static analysis and strong typing
* Support for language extensions without breaking existing tools
	* The AST should be able to cast to any normal version of Lua
	* The project should either:
		* Leverage Rust's type system (non-exhaustive patterns, especially) to guarantee that tools can be recompiled with forks of this project with zero changes.
		* Or, use a technique similar to an Entity Component System to implement extended tokens and AST nodes.

## Contributing
Contributions are welcome -- there is a lot of work to be done!

Mab supports Rust 1.26 stable and newer.

There is already a fairly sizable test suite implemented as a "parse by example" system. Test file inputs are located in `parse_examples/source`.

The test runner (`cargo test`) will read, tokenize, and parse these source files and check the last-known good results folder (`parse_examples/results`) to see if they match what they did before.

If you're making changes that modify the parser's AST, delete the corresponding serialized token list and AST JSON files. When you run the test runner next, it will generate files that should be manually reviewed and submitted alongside your change. Git's diff viewer can help identify if what was changed was intentional.

Be careful with line endings when developing on Windows. The repository has a `.editorconfig` file as well as a `.gitattributes` file to try to guarantee that all Lua files have `LF` line endings as opposed to `CRLF`. Checking in a parse by example token list with `CRLF` line endings baked into it will cause CI to fail.

## License
This project is available under the terms of The Mozilla Public License, version 2.0. Details are available in [LICENSE](LICENSE).