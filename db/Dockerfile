FROM postgres:15

# pgvector のビルドに必要なパッケージをインストール
RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    postgresql-server-dev-15

# pgvector をインストール
RUN git clone --branch v0.5.1 https://github.com/pgvector/pgvector.git && \
    cd pgvector && \
    make && \
    make install

# 初期化スクリプトをコピー
COPY ./init/init.sql /docker-entrypoint-initdb.d/
