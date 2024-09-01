---
title: "Analysis of Type--Driven approach to systems programming: Implementation of OpenGL library for Rust"
author: "Mikołaj Depta"
date: "03.09.2024"
theme: "black"
github: "https://github.com/mikiSpoko200/gpu-bulwark"
---
# Agenda
- Introduction (5min)
  * Why Rust
  * Why OpenGL
  * Goals and key features
  * Overview of the project
- Code Examples (10min)
  * Live demo: `gpu-bulwark` vs raw OpenGL in `C` / `C++`
- Conclusions (5min)
  * What was achieved
  * What can be improved
  * Future plans

---
## Introduction - Why Rust

Rust is a very inspiring language to me on so many levels.
It seamlessly interleaves the elegance of functional languages,
with clear and predictable view of computers resources comparable to C's.

Rust is minimalistic yet very deep. It may make simple things complex, but as a reward it makes complex things manageable.
Understanding of rust's ownership model and data oriented design, 
allows one to gain unique perspective on how computer systems interact and function.

---
## Introduction - Why OpenGL

OpenGL is a specification for hardware accelerated computer graphics.
Its well-understood, broadly supported and most importantly - simple.

Idea to write a wrapper for OpenGL originated during the computer graphics course.
Whilst writing in Rust I was natual

This is how the idea behind `gpu-bulwark` started.

## Introduction - Goals

Our goals with this study were twofold:
1. Explore type-driven design in Rust:
  - Determine benefits and downsides of such an approach to systems programming
  - Ascertain feasibility of developing software that way
  - Identify common patterns for type-driven design
2. Create a minimalistic OpenGL wrapper:
  - Remain as close to original specification as possible, simply provide typing information and object oriented syntax.
  - Implement the most essential functionality as it would be unfeasible to cover entirety of the spec without diminishing quality
  - Prevent as many ill-formed programs at compile-time as possible

## Introduction -- Key Features

- Extensible typing scheme for shaders, programs and vertices. Type level, LISP-like lists of types represent 
- 

# Examples

## Examples - Program building

### Vertex Specification

### Attributes

### Invalid format for input glsl shader

## Expansions 

### `gpu-bulwark` + `rust-gpu` 

### RT - CT error detection

### Two versions of the API - direct state + pre 4.5 binders.

### Unassigned attribute

### Invalid vertex data

### 