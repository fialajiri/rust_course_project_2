-- Password is 'password123' hashed with bcrypt
INSERT INTO users (username, email, password_hash) VALUES
('alice', 'alice@test.com', '$2b$10$/ssaReyA9pP6H7UmH2rQ9Owqkb/2nAZj9ctcpCP7YyaJwjXhU3iO2'),
('bob', 'bob@test.com', '$2b$10$/ssaReyA9pP6H7UmH2rQ9Owqkb/2nAZj9ctcpCP7YyaJwjXhU3iO2'),
('carol', 'carol@test.com', '$2b$10$/ssaReyA9pP6H7UmH2rQ9Owqkb/2nAZj9ctcpCP7YyaJwjXhU3iO2'); 