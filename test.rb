require 'net/smtp'

msgstr = <<END_OF_MESSAGE
From: Your Name <your@mail.address>
To: Destination Address <someone@example.com>
Subject: test message
Date: Sat, 23 Jun 2001 16:26:43 +0900
Message-Id: <unique.message.id.string@example.com>

This is a test message.
END_OF_MESSAGE

Net::SMTP.start('127.0.0.1', 2525, 'mail.from.domain') do |smtp|
  smtp.send_message msgstr, 'from@ntecs.de', 'to@ntecs.de'
end
