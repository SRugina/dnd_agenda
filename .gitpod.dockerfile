FROM gitpod/workspace-postgres

RUN sudo apt update \
 && sudo apt install npx \