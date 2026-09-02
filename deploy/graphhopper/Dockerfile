FROM eclipse-temurin:21-jre-jammy AS download

ARG GRAPHHOPPER_VERSION=10.2
ARG GRAPHHOPPER_SHA256=ead763749c395ea0cc45b3fd10092d6de84d62fb5cbaf7d416ba6cc142c5716c
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && curl --fail --location --output /graphhopper.jar \
       "https://repo1.maven.org/maven2/com/graphhopper/graphhopper-web/${GRAPHHOPPER_VERSION}/graphhopper-web-${GRAPHHOPPER_VERSION}.jar" \
    && echo "${GRAPHHOPPER_SHA256}  /graphhopper.jar" | sha256sum --check --strict \
    && rm -rf /var/lib/apt/lists/*

FROM eclipse-temurin:21-jre-jammy
RUN groupadd --system --gid 10001 graphhopper \
    && useradd --system --uid 10001 --gid graphhopper --home-dir /app graphhopper \
    && mkdir -p /app /data/graph-cache /data/srtm \
    && chown -R graphhopper:graphhopper /app /data
COPY --from=download --chown=graphhopper:graphhopper /graphhopper.jar /app/graphhopper.jar
COPY --chown=graphhopper:graphhopper config.yml /app/config.yml
COPY --chown=graphhopper:graphhopper entrypoint.sh /app/entrypoint.sh
USER graphhopper
WORKDIR /app
EXPOSE 8989 8990
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["server"]
