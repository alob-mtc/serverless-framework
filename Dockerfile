# Use an official Python runtime as a parent image
FROM python:3.8

# Set the working directory in the container
WORKDIR /usr/src/app

# When running the container, Python will be invoked
ENTRYPOINT ["python", "-c"]
