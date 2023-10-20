pub const SELECT: &str = "SELECT
t.merkle_root_hex,
array_agg(leaves.content) AS leaves
FROM personal.user u
INNER JOIN personal.transaction t
ON t.user_id = u.id
INNER JOIN (
   SELECT
     content,
     user_id
 FROM
 personal.leaf
 ORDER BY created_at ASC
) leaves
ON leaves.user_id = u.id
WHERE u.id=$1
GROUP BY t.merkle_root_hex;";