as_docker_cache_key () {
  DOCKER_REF="$1"
  if ! grep -q ":" <<< "$DOCKER_REF";
  then
    >&2 echo "as_docker_cache_key >> argument '$DOCKER_REF' doesn't seem to be reference to a docker image, because it doesn't contain colon"
    exit 1
  fi;
  echo "$DOCKER_REF" | tr ':' '-'
}
