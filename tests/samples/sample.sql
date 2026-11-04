-- 한 줄 주석
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

/* 블록 주석 */
INSERT INTO users (id, name) VALUES (1, '홍길동'), (2, 'Don');

SELECT u.id, u.name, COUNT(o.id) AS order_count
FROM users u
LEFT JOIN orders o ON o.user_id = u.id
WHERE u.id > 0 AND u.name LIKE 'D%'
GROUP BY u.id, u.name
ORDER BY order_count DESC
LIMIT 10;

UPDATE users SET name = 'Don Editor' WHERE id = 2;
DELETE FROM users WHERE id = 3;
