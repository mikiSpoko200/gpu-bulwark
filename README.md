# Analysis of Type-Driven approach to systems programming: Implementation of OpenGL library for Rust

## Abstract

For the past few years in the software development industry there has been a growing interest in strongly typed languages. It manifests itself in emergence of brand-new technologies in which strong type systems were one of the core founding principles or in changes introduced to existing languages. The most common examples of modern languages with powerful type systems are TypeScript as an alternative to JavaScript in the world of web development or Rust in domain of systems programming in place of C and C++. More mature languages also had their type systems revised, for example in C# 8 explicit type nullability annotations were introduced, or even dynamically typed Python has seen major improvements to its type annotation system.

This study - an implementation of the OpenGL graphics API wrapper library for Rust - will attempt to demonstrate how Rust's type system can be utilized to improve low--level software safety and maintainability as well as how it affects API design and codebase structure.
	
## Goals

This work aims to achieve several things:
- provide an ergonomic, efficient, and low-cost OpenGL wrapper library for Rust
- identify common patterns present in Type-Driven design in Rust
- determine pros and cons of such an approach to systems programming
- measure performance characteristics and compare the result against existing alternatives based on C and C++ languages.
