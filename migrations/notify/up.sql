CREATE TABLE notify (
	channel UNSIGNED BIG INT NOT NULL,
	-- 00 none/undefined          00 my
	-- 01 movies                  01 all
	-- 10 shows                   10 undefined
	-- 11 all               11 undefined
	type INT(4) DEFAULT 12 NOT NULL,
	-- discord_id for my
	-- null for all
	data UNSIGNED BIG INT,
	CONSTRAINT PK_notify PRIMARY KEY (channel, type, data),
	CONSTRAINT FK_users FOREIGN KEY (data) REFERENCES users(discord_id)
)

