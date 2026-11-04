# 한 줄 주석
require "json"

module Greeting
  class Person
    attr_reader :name

    def initialize(name, age = 20)
      @name = name
      @age = age
    end

    def to_s
      "이름: #{@name}, 나이: #{@age}"
    end
  end
end

people = [Greeting::Person.new("홍길동"), Greeting::Person.new("Don", 30)]
people.each do |person|
  puts person if person.name != nil
end

hash = { key: :value, "문자열" => 3.14 }
puts hash.to_json
