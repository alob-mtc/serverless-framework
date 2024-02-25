# code Runner

### A Remote Code Execution Engine

Code-Runner is a simple piece of software that enables the execution of code snippets in isolated environments. It accepts a set of code, written in supported languages (Python or JavaScript), and executes this code securely within a Docker container. The result of this execution is then formatted and returned as output.

### Process Flow:

```
+---------------------------+
| Input Code                |
| [Python, JavaScript]      |
+---------------------------+
              |
              | Code Submission
              V
+-------------------------------+
| Spin Up Docker Container      |
| From Pre-defined Docker Image |
+-------------------------------+
              |
              | Container Setup
              V
+-------------------------------+
| Pass Code to Docker Engine    |
+-------------------------------+
              |
              | Execution Start
              |<----------------------------+
              V                             |
+-------------------------------+           |
| Execute Code Inside Container |           |
+-------------------------------+           | Monitor Execution
              |                             | Terminate if Time Limit Exceeded
              | Execution Complete          |
              V                             |
+-------------------------------+           |
| Capture Execution Output      |<----------+
+-------------------------------+
              |
              | Output Processing
              V
+-------------------------------+
| Format and Return Output      |
+-------------------------------+
              |
              | Cleanup Process
              V
+-------------------------------+
| Remove Docker Container       |
+-------------------------------+

```
