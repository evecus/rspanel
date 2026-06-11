FROM alpine:3.19

ARG TARGETARCH

WORKDIR /app

RUN apk add --no-cache ca-certificates tzdata

RUN mkdir -p /app/data && chmod 755 /app/data

# 根据目标架构自动选择预编译的静态二进制
COPY rspanel-linux-${TARGETARCH}-bin /app/rspanel
RUN chmod +x /app/rspanel

VOLUME ["/app/data"]

EXPOSE 3088

ENTRYPOINT ["/app/rspanel"]
