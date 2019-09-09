FROM scratch
COPY ./ohx-app /app
ENTRYPOINT ["/app"]
