# AGENTS.md

## Introduction

This document outlines the rules and guidelines for AI agents or modes operating within the MSMS Dev Tools, a Tauri-based desktop application designed for developer productivity. The application integrates with various services such as AWS, Kubernetes, and provides tools for HTTP requests, file management, shell commands, and dock progress indicators. It employs a hybrid architecture combining a Rust backend for native system interactions and a Next.js frontend for the user interface, promoting modularity, event-driven communication, and clear separation of concerns to ensure maintainability and extensibility.

## Tech Stack Overview

- **Backend (Rust/Tauri)**: Built with Tauri 2, leveraging Tokio for asynchronous operations, Serde for serialization/deserialization, Reqwest for HTTP client functionality, AWS SDK for S3 interactions, Kube for Kubernetes management, and Tracing for logging and debugging.
- **Frontend (Next.js/React)**: Utilizes Next.js 14 with React 18, Radix UI components for accessible UI elements, Tailwind CSS for styling, Zustand for state management, and Monaco Editor for code editing capabilities.
- **Integrations**: Includes AWS profile management with SSO support, Kubernetes configuration and logging, HTTP request handling, shell command execution, and macOS-specific features like dock progress and vibrancy.
- **Build and Development Tools**: Employs NPM for frontend builds, Cargo for Rust compilation, and Tauri CLI for packaging and development.

## Architecture Patterns

- **Modular Design**: The codebase is structured into distinct modules (e.g., services for AWS, Kubernetes, HTTP) to promote reusability and separation of concerns, with each module handling specific functionalities independently.
- **State Management**: Backend state is managed using `Arc<Mutex<AppState>>` for thread-safe shared state, while the frontend uses Zustand for reactive state handling, synchronized via Tauri commands and events.
- **Event-Driven Communication**: Utilizes Tauri emitters for bidirectional communication between the frontend and backend, enabling real-time updates such as window state changes or navigation events.
- **Error Handling**: Implements custom error types (e.g., ApiError, AwsError, KubeError) with Result-based propagation, ensuring robust error reporting and logging throughout the application.
- **Tauri Capabilities**: Leverages Tauri plugins for shell access, dialogs, global shortcuts, drag-and-drop, and system information, while adhering to platform-specific features like macOS private APIs for enhanced UI effects.

## Mode-Specific Rules

### Architect Mode
- Focus on high-level planning, design, and strategy for tasks involving system architecture or feature implementation.
- Restrict edits to markdown files (e.g., .md) for documentation and planning purposes.
- Use analysis tools to gather context, create detailed todo lists for complex tasks, and emphasize architectural decisions such as module separation and integration patterns.
- Avoid direct code modifications; instead, propose designs that maintain the hybrid Rust/Next.js structure and event-driven patterns.

### Code Mode
- Handle implementation of features, bug fixes, and code refactoring across Rust and TypeScript/JavaScript files.
- Use targeted editing tools like `apply_diff` for surgical changes to ensure compatibility with the existing modular design.
- Respect file restrictions and ensure changes align with Tauri capabilities, such as command handlers and plugin integrations.
- Prioritize separation of concerns, ensuring backend services remain independent and frontend components are reactive and event-responsive.

### Ask Mode
- Provide explanations, documentation, and answers to technical questions about the codebase, architecture, or integrations.
- Do not perform any file edits or modifications; focus solely on informational responses.
- Analyze code patterns, dependencies, and best practices without altering the project state.

### Debug Mode
- Specialize in troubleshooting issues, investigating errors, and diagnosing problems in both backend and frontend.
- Add logging, analyze stack traces, and use debugging tools to identify root causes, such as async handling in Rust or state synchronization in React.
- If frontend debugging is required, utilize browser interactions for verification, but ensure compatibility with Tauri’s webview environment.

### Orchestrator Mode
- Coordinate complex, multi-step projects by breaking them down into subtasks and managing workflows across different domains.
- Dynamically switch modes as needed (e.g., from Architect to Code) to handle planning, implementation, and verification phases.
- Maintain oversight of the hybrid structure, ensuring service integrations (e.g., AWS, Kubernetes) are handled efficiently and state management remains consistent.

## Guidelines for Hybrid Rust/Next.js Structure, Service Integrations, State Management, Error Handling, and Best Practices

- **Hybrid Rust/Next.js Structure**: The backend (Rust) should handle native operations like file system access, AWS/Kubernetes interactions, and system-level commands, while the frontend (Next.js) manages UI rendering, user interactions, and client-side logic. Communication must occur via Tauri commands and events to maintain separation.
- **Service Integrations**: For AWS and Kubernetes, use async clients with proper error handling and SSO management for AWS profiles. Ensure integrations are modular and do not tightly couple with UI components.
- **State Management**: Synchronize state between backend and frontend using Tauri emitters for events like window resizing or navigation. Use immutable updates in Zustand for frontend state and thread-safe structures in Rust to avoid race conditions.
- **Error Handling**: Propagate errors through custom Result types, log them appropriately with Tracing, and display user-friendly messages in the frontend without exposing sensitive details.
- **Terminal Usage**: When executing commands via agents, prepend tool binary locations to PATH (e.g., `export PATH="$HOME/.config/cargo/bin:$PATH"` for cargo) to ensure proper access, as demonstrated in workflows like running "cargo check" after PATH updates.
- **Best Practices**: Maintain modular design by keeping services independent, use event-driven patterns for updates, ensure Tauri compatibility by testing with plugins, and follow Rust/Next.js conventions for code style and performance. Regularly review for separation of concerns and optimize for cross-platform compatibility.

This document serves as a reference for maintaining consistency and efficiency in development activities within the MSMS Dev Tools project.