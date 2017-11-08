module ActiveRecord
  RSpec.describe Attribute do
    let(:type) { Type::Value.new }

    specify "it cannot be subclassed in Ruby" do
      expect { Class.new(Attribute) }.to raise_error(/cannot be subclassed/)
    end

    specify "from_database + read type casts from database" do
      allow(type).to receive(:deserialize).and_return("type cast from database")
      attribute = Attribute.from_database(nil, "a value", type)

      type_cast_value = attribute.value

      expect(type_cast_value).to eq("type cast from database")
    end

    specify "from_user + read type casts from user" do
      allow(type).to receive(:cast).and_return("type cast from user")
      attribute = attribute_from_user(nil, "a value", type)

      type_cast_value = attribute.value

      expect(type_cast_value).to eq("type cast from user")
    end

    specify "reading memoizes the value" do
      allow(type).to receive(:deserialize) { "from the database".dup }
      attribute = Attribute.from_database(nil, "whatever", type)

      type_cast_value = attribute.value
      second_read = attribute.value

      expect(type_cast_value).to eq("from the database")
      expect(second_read).to equal(type_cast_value)
    end

    specify "reading memoizes falsy values" do
      allow(type).to receive(:deserialize).and_return(false)
      attribute = Attribute.from_database(nil, "whatever", type)

      attribute.value
      attribute.value

      expect(type).to have_received(:deserialize).once
    end

    specify "read_before_typecast returns the given value" do
      attribute = Attribute.from_database(nil, "raw value", type)

      raw_value = attribute.value_before_type_cast

      expect(raw_value).to eq("raw value")
    end

    specify "from_database + read_for_database type casts to and from database" do
      allow(type).to receive(:deserialize).and_return("read from database")
      allow(type).to receive(:serialize).and_return("ready for database")
      attribute = Attribute.from_database(nil, "whatever", type)

      serialize = attribute.value_for_database

      expect(serialize).to eq("ready for database")
    end

    specify "from_user + read_for_database type casts from the user to the database" do
      allow(type).to receive(:cast).and_return("read from user")
      allow(type).to receive(:serialize).and_return("ready for database")
      attribute = attribute_from_user(nil, "whatever", type)

      serialize = attribute.value_for_database

      expect(serialize).to eq("ready for database")
    end

    specify "duping dups the value" do
      allow(type).to receive(:deserialize) { "type cast" }
      attribute = Attribute.from_database(nil, "a value", type)

      value_from_orig = attribute.value
      value_from_clone = attribute.dup.value
      value_from_orig << " foo"

      expect(value_from_orig).to eq("type cast foo")
      expect(value_from_clone).to eq("type cast")
    end

    specify "duping does not dup the value if it is not dupable" do
      allow(type).to receive(:deserialize).and_return(false)
      attribute = Attribute.from_database(nil, "a value", type)

      expect(attribute.dup.value).to equal(attribute.value)
    end

    specify "duping does not eagerly type cast if we have not yet type cast" do
      expect(type).not_to receive(:deserialize)
      attribute = Attribute.from_database(nil, "a value", type)
      attribute.dup
    end

    class MyType
      def cast(value)
        value + " from user"
      end

      def deserialize(value)
        value + " from database"
      end

      def assert_valid_value(*)
      end
    end

    specify "with_value_from_user returns a new attribute with the value from the user" do
      old = Attribute.from_database(nil, "old", MyType.new)
      new = old.with_value_from_user("new")

      expect(old.value).to eq("old from database")
      expect(new.value).to eq("new from user")
    end

    specify "with_value_from_database returns a new attribute with the value from the database" do
      old = attribute_from_user(nil, "old", MyType.new)
      new = old.with_value_from_database("new")

      expect(old.value).to eq("old from user")
      expect(new.value).to eq("new from database")
    end

    specify "uninitialized attributes yield their name if a block is given to value" do
      block = proc { |name| name.to_s + "!" }
      foo = Attribute.uninitialized(:foo, nil)
      bar = Attribute.uninitialized(:bar, nil)

      expect(foo.value(&block)).to eq("foo!")
      expect(bar.value(&block)).to eq("bar!")
    end

    specify "uninitialized attributes have no value" do
      expect(Attribute.uninitialized(:foo, nil).value).to be_nil
    end

    specify "attributes equal other attributes with the same constructor arguments" do
      first = Attribute.from_database(:foo, 1, Type::Integer.new)
      second = Attribute.from_database(:foo, 1, Type::Integer.new)
      expect(second).to eq(first)
    end

    specify "attributes do not equal attributes with different names" do
      first = Attribute.from_database(:foo, 1, Type::Integer.new)
      second = Attribute.from_database(:bar, 1, Type::Integer.new)
      expect(second).not_to eq(first)
    end

    specify "attributes do not equal attributes with different types" do
      first = Attribute.from_database(:foo, 1, Type::Integer.new)
      second = Attribute.from_database(:foo, 1, Type::Float.new)
      expect(second).not_to eq(first)
    end

    specify "attributes do not equal attributes with different values" do
      first = Attribute.from_database(:foo, 1, Type::Integer.new)
      second = Attribute.from_database(:foo, 2, Type::Integer.new)
      expect(second).not_to eq(first)
    end

    specify "attributes do not equal attributes of other classes" do
      first = Attribute.from_database(:foo, 1, Type::Integer.new)
      second = attribute_from_user(:foo, 1, Type::Integer.new)
      expect(second).not_to eq(first)
      expect(first).not_to eq(1)
    end

    specify "an attribute has not been read by default" do
      attribute = Attribute.from_database(:foo, 1, Type::Value.new)
      expect(attribute).not_to have_been_read
    end

    specify "an attribute has been read when its value is calculated" do
      attribute = Attribute.from_database(:foo, 1, Type::Value.new)
      attribute.value
      expect(attribute).to have_been_read
    end

    specify "an attribute is not changed if it hasn't been assigned or mutated" do
      attribute = Attribute.from_database(:foo, 1, Type::Value.new)

      expect(attribute).not_to be_changed
    end

    specify "an attribute is changed if it's been assigned a new value" do
      attribute = Attribute.from_database(:foo, 1, Type::Value.new)
      changed = attribute.with_value_from_user(2)

      expect(changed).to be_changed
    end

    specify "an attribute is not changed if it's assigned the same value" do
      attribute = Attribute.from_database(:foo, 1, Type::Value.new)
      unchanged = attribute.with_value_from_user(1)

      expect(unchanged).not_to be_changed
    end

    specify "an attribute can not be mutated if it has not been read,
        and skips expensive calculations" do
      type_which_raises_from_all_methods = Object.new
      attribute = Attribute.from_database(:foo, "bar", type_which_raises_from_all_methods)

      expect(attribute).not_to be_changed_in_place
      expect(attribute).not_to be_changed
    end

    specify "an attribute is changed if it has been mutated" do
      attribute = Attribute.from_database(:foo, "bar", Type::String.new)
      attribute.value << "!"

      expect(attribute).to be_changed_in_place
      expect(attribute).to be_changed
    end

    specify "an attribute can forget its changes" do
      attribute = Attribute.from_database(:foo, "bar", Type::String.new)
      changed = attribute.with_value_from_user("foo")
      forgotten = changed.forgetting_assignment

      expect(changed).to be_changed # sanity check
      expect(forgotten).not_to be_changed
    end

    specify "with_value_from_user validates the value" do
      type = Type::Value.new
      type.define_singleton_method(:assert_valid_value) do |value|
        if value == 1
          raise ArgumentError
        end
      end

      attribute = Attribute.from_database(:foo, 1, type)
      expect(attribute.value).to eq(1)
      expect(attribute.with_value_from_user(2).value).to eq(2)
      expect { attribute.with_value_from_user(1) }.to raise_error(ArgumentError)
    end

    specify "with_type preserves mutations" do
      attribute = Attribute.from_database(:foo, "".dup, Type::Value.new)
      attribute.value << "1"

      expect(attribute.with_type(Type::Integer.new).value).to eq(1)
    end

    def attribute_from_user(name, value, type)
      Attribute.from_user(name, value, type, Attribute.uninitialized(name, type))
    end
  end
end
