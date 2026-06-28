CREATE UNIQUE INDEX links_original_url_md5_idx
    ON links (md5(original_url));
