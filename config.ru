require 'json'
require 'net/http'

use Rack::Static, :root => 'public'

use Rack::Auth::Basic do |username, password|
  ENV['HTTP_BASIC_AUTH_PASSWORD'].present? &&
    username == 'user' &&
    password == ENV['HTTP_BASIC_AUTH_PASSWORD']
end

app = lambda do |env|
  request_uri = env['REQUEST_URI']

  if request_uri == '/health'
    return [200, { 'content-type' => 'text/plain' }, [Time.now.to_i.to_s]]
  end

  body = lambda do |stream|
    puts "\nFetching: #{request_uri}"
    puts "---------\n\n"

    uri = URI('https://api.openai.com/v1/chat/completions')

    request = Net::HTTP::Post.new(uri)
    request['content-type'] = 'application/json'
    request['authorization'] = "Bearer #{ENV['OPENAI_API_KEY']}"
    request.body = JSON.dump({
      model: 'gpt-3.5-turbo',
      stream: true,
      messages: [
        {
          role: 'system',
          content: <<~HEREDOC,
            Output a valid HTML document for the webpage that could be located at the URL path provided by the user. Include general navigation anchor tags as well as relative anchor tags to other related pages. Include a minimal amount of inline styles to improve the look of the page. Make the text content quite long with a decent amount of interesting content. Do not use any dummy text on the page.

            Start the reponse with the following exact characters:

            <!doctype html>
            <html>
          HEREDOC
        },
        { role: 'user', content: request_uri },
      ],
    })

    Net::HTTP.start(uri.host, uri.port, use_ssl: true) do |http|
      http.request(request) do |response|
        response.read_body do |chunk|
          chunk.sub(/\Adata: /, '').sub(/\n\n\z/, '').split("\n\ndata: ").each do |data|
            break if data == '[DONE]'

            content = JSON.parse(data).dig('choices')&.first.dig('delta', 'content')

            next if content.nil?

            stream.write(content)

            # Debug
            print(content)
          end
        end
      end
    end
  ensure
    stream.close
    puts
  end

  [200, { 'content-type' => 'text/html' }, body]
end

run app
