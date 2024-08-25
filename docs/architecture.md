# Gam3du architecture

This document describes the components of this project and how they play together.

## Framework

The framework (located under `framework`) covers the fundamental building block of every application. It is agnostic to any specific [engine](#engines), scripting language or [API](#apis) but enables a developer to integrate any of these.

Some of the main features of the framework:

- locating and loading resources
- abstracting the platform (graphics, sound, input, operating system, ...)
- starting the scripting runtimes (e.g. Python VM, WASM, ...)
- loading an engine

## Engines

## APIs
