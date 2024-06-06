# Use a smaller base image
FROM debian:buster-slim

# Copy the pre-built executable
COPY target/release/translate_server /usr/local/bin/translate_server

# Set the environment variables
ENV OPENAI_API_KEY=your_openai_api_key

# Expose the port that the application will run on
EXPOSE 3000

# Run the application
CMD ["translate_server"]