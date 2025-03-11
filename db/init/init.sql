-- pgvector 拡張をインストール
CREATE EXTENSION IF NOT EXISTS vector;

-- ベクトルを格納するテーブル
CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    embedding VECTOR(1024) NOT NULL  -- 384次元のベクトル（必要に応じて次元数を変更）
);

-- 効率的な検索のためのインデックス作成
CREATE INDEX ON documents USING ivfflat (embedding vector_l2_ops) WITH (lists = 100);

-- サンプルデータ挿入（ここではランダムベクトルを使用）
INSERT INTO documents (title, embedding)
SELECT
    'Document ' || i,
    random_vector(1024)
FROM generate_series(1, 1000) AS i;
