
# Introduction 

Rust's type system for OpenGL
`gpu-bulwark` 

---

## Why?

---
## Rust

Rust is a modern, compiled, general-purpose programming languge.
Its a new way to do systems programming.
Aim of this study was to, demonstrate a wide range of 


---
## OpenGL

OpenGL is excellent API to start graphics programming due to its
simple view of the GPU, but its programming interface is very dated.

# Computer graphics primer

Goal: Display 3D geometry onto 2D surface of a computer's screen.



## Graphics pipeline

It can be broadly divided into to conceptual halves:

**geometry processing** `->` discretization `->` **pixel processing**

- some parts are programmable 
- some allow limited configuration, but are mostly performed via a predefined process 

---
## Terminology

- shader - a routine which runs on the GPU, that implements programmable stages of the graphics pipeline.
- fixed-function - term commonly used to describe parts of graphics pipeline which are not freely programmable, but rather configurable to a limited degree.

---
## Geometric data

Geometric data is specified using abstract notion of a vertex in a mesh representing
the surface of a 3D object.

---
## Rasterization

Geometric shapes must at some point be converted into discrete set of pixels.

This process of conversion, from continous geometric data
into discrete color samples is called rasterization.

Fixed function stages interpolate vertex data across prmitive's surface.
This collection of interpolated data is called a **fragment**.

---
## Fragment processing

Fragments are subsequently processed by a fragment shader, to obtain color values.
There per pixel lighting computations using interpolated surface normals,
materials are applied using texture mapping, and different post processing effects 
may be applied.

# OpenGL API

## Objects

Objects are how OpenGL represents GPU resources.

There are number of objects, but most these matter the most for our purposes:
- buffers
- vertex arrays
- shaders
- programs
- textures

## Buffers

Buffers provide the ability to explicitly allocated memory on the GPU.
Buffer objects have different targets -- which symbolise different predefined use cases,
which are listed by the spec.

## Vertex Arrays

## Shaders and Program objects

To start, shader source code is to loaded from a files, uploaded to OpenGL, and compiled.
With required minimum of vertex and fragment shaders they can be attached to a program.

## Program

Programs are 

# Examples

## Program building

## Vertex Specification

## Attributes

### Invalid format for input glsl shader


## Expansions 

### `gpu-bulwark` + `rust-gpu` 

### RT - CT error detection

### Two versions of the API - direct state + pre 4.5 binders.

### Unassigned attribute

### Invalid vertex data

### 