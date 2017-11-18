module ActiveModel
  RSpec.describe AttributeSet do
    specify "building a new set from raw attributes" do
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::Float.new)
      attributes = builder.build_from_database(foo: "1.1", bar: "2.2")

      expect(attributes[:foo].value).to eq(1)
      expect(attributes[:bar].value).to eq(2.2)
      expect(attributes[:foo].name).to eq(:foo)
      expect(attributes[:bar].name).to eq(:bar)
    end

    specify "building with string keys" do
      builder = AttributeSet::Builder.new("foo" => Type::Integer.new, bar: Type::Float.new)
      attributes = builder.build_from_database(foo: "1.1", "bar" => "2.2")

      expect(attributes["foo"].value).to eq(1)
      expect(attributes[:bar].value).to eq(2.2)
      expect(attributes[:foo].name).to eq("foo")
      expect(attributes["bar"].name).to eq(:bar)
    end

    specify "building with custom types" do
      builder = AttributeSet::Builder.new(foo: Type::Float.new)
      attributes = builder.build_from_database({ foo: "3.3", bar: "4.4" }, { bar: Type::Integer.new })

      expect(attributes[:foo].value).to eq(3.3)
      expect(attributes[:bar].value).to eq(4)
    end

    specify "[] returns a null object" do
      builder = AttributeSet::Builder.new(foo: Type::Float.new)
      attributes = builder.build_from_database(foo: "3.3")

      expect(attributes[:foo].value_before_type_cast).to eq("3.3")
      expect(attributes[:bar].value_before_type_cast).to be_nil
      expect(attributes[:bar].name).to eq(:bar)
    end

    specify "duping creates a new hash, but does not dup the attributes" do
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::String.new)
      attributes = builder.build_from_database(foo: 1, bar: "foo")

      # Ensure the type cast value is cached
      attributes[:foo].value
      attributes[:bar].value

      duped = attributes.dup
      duped.write_from_database(:foo, 2)
      duped[:bar].value << "bar"

      expect(attributes[:foo].value).to eq(1)
      expect(duped[:foo].value).to eq(2)
      expect(attributes[:bar].value).to eq("foobar")
      expect(duped[:bar].value).to eq("foobar")
    end

    specify "deep_duping creates a new hash and dups each attribute" do
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::String.new)
      attributes = builder.build_from_database(foo: 1, bar: "foo")

      # Ensure the type cast value is cached
      attributes[:foo].value
      attributes[:bar].value

      duped = attributes.deep_dup
      duped.write_from_database(:foo, 2)
      duped[:bar].value << "bar"

      expect(attributes[:foo].value).to eq(1)
      expect(duped[:foo].value).to eq(2)
      expect(attributes[:bar].value).to eq("foo")
      expect(duped[:bar].value).to eq("foobar")
    end

    specify "freezing cloned set does not freeze original" do
      attributes = AttributeSet.new({})
      clone = attributes.clone

      clone.freeze

      expect(clone).to be_frozen
      expect(attributes).not_to be_frozen
    end

    specify "to_hash returns a hash of the type cast values" do
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::Float.new)
      attributes = builder.build_from_database(foo: "1.1", bar: "2.2")

      expect(attributes.to_hash).to eq({ foo: 1, bar: 2.2 })
      expect(attributes.to_h).to eq({ foo: 1, bar: 2.2 })
    end

    specify "to_hash maintains order" do
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::Float.new)
      attributes = builder.build_from_database(foo: "2.2", bar: "3.3")

      attributes[:bar]
      hash = attributes.to_h

      expect(hash.to_a).to eq([[:foo, 2], [:bar, 3.3]])
    end

    specify "values_before_type_cast" do
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::Integer.new)
      attributes = builder.build_from_database(foo: "1.1", bar: "2.2")

      expect(attributes.values_before_type_cast).to eq({ foo: "1.1", bar: "2.2" })
    end

    specify "known columns are built with uninitialized attributes" do
      attributes = attributes_with_uninitialized_key
      expect(attributes[:foo]).to be_initialized
      expect(attributes[:bar]).not_to be_initialized
    end

    specify "uninitialized attributes are not included in the attributes hash" do
      attributes = attributes_with_uninitialized_key
      expect(attributes.to_hash).to eq({ foo: 1 })
    end

    specify "uninitialized attributes are not included in keys" do
      attributes = attributes_with_uninitialized_key
      expect(attributes.keys).to eq([:foo])
    end

    specify "uninitialized attributes return false for key?" do
      attributes = attributes_with_uninitialized_key
      expect(attributes.key?(:foo)).to be
      expect(attributes.key?(:bar)).not_to be
    end

    specify "unknown attributes return false for key?" do
      attributes = attributes_with_uninitialized_key
      expect(attributes.key?(:wibble)).not_to be
    end

    specify "fetch_value returns the value for the given initialized attribute" do
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::Float.new)
      attributes = builder.build_from_database(foo: "1.1", bar: "2.2")

      expect(attributes.fetch_value(:foo)).to eq(1)
      expect(attributes.fetch_value(:bar)).to eq(2.2)
    end

    specify "fetch_value returns nil for unknown attributes" do
      attributes = attributes_with_uninitialized_key
      expect(attributes.fetch_value(:wibble) { "hello" }).to be_nil
    end

    specify "fetch_value returns nil for unknown attributes when types has a default" do
      types = Hash.new(Type::Value.new)
      builder = AttributeSet::Builder.new(types)
      attributes = builder.build_from_database

      expect(attributes.fetch_value(:wibble) { "hello" }).to be_nil
    end

    specify "fetch_value uses the given block for uninitialized attributes" do
      attributes = attributes_with_uninitialized_key
      value = attributes.fetch_value(:bar) { |n| n.to_s + "!" }
      expect(value).to eq("bar!")
    end

    specify "fetch_value returns nil for uninitialized attributes if no block is given" do
      attributes = attributes_with_uninitialized_key
      expect(attributes.fetch_value(:bar)).to be_nil
    end

    specify "the primary_key is always initialized" do
      default_attributes = { foo: Attribute.from_database(:foo, nil, nil) }
      builder = AttributeSet::Builder.new({ foo: Type::Integer.new }, default_attributes)
      attributes = builder.build_from_database

      expect(attributes.key?(:foo)).to be
      expect(attributes.keys).to eq([:foo])
      expect(attributes[:foo]).to be_initialized
    end

    class MyType
      def cast(value)
        return if value.nil?
        value + " from user"
      end

      def deserialize(value)
        return if value.nil?
        value + " from database"
      end

      def assert_valid_value(*)
      end
    end

    specify "write_from_database sets the attribute with database typecasting" do
      builder = AttributeSet::Builder.new(foo: MyType.new)
      attributes = builder.build_from_database

      expect(attributes.fetch_value(:foo)).to be_nil

      attributes.write_from_database(:foo, "value")

      expect(attributes.fetch_value(:foo)).to eq("value from database")
    end

    specify "write_from_user sets the attribute with user typecasting" do
      builder = AttributeSet::Builder.new(foo: MyType.new)
      attributes = builder.build_from_database

      expect(attributes.fetch_value(:foo)).to be_nil

      attributes.write_from_user(:foo, "value")

      expect(attributes.fetch_value(:foo)).to eq("value from user")
    end

    def attributes_with_uninitialized_key
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::Float.new)
      builder.build_from_database(foo: "1.1")
    end

    specify "freezing doesn't prevent the set from materializing" do
      builder = AttributeSet::Builder.new(foo: Type::String.new)
      attributes = builder.build_from_database(foo: "1")

      attributes.freeze
      expect(attributes.to_hash).to eq({ foo: "1" })
    end

    specify "#accessed_attributes returns only attributes which have been read" do
      builder = AttributeSet::Builder.new(foo: Type::Value.new, bar: Type::Value.new)
      attributes = builder.build_from_database(foo: "1", bar: "2")

      expect(attributes.accessed).to eq([])

      attributes.fetch_value(:foo)

      expect(attributes.accessed).to eq([:foo])
    end

    specify "#map returns a new attribute set with the changes applied" do
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::Integer.new)
      attributes = builder.build_from_database(foo: "1", bar: "2")
      new_attributes = attributes.map do |attr|
        attr.with_cast_value(attr.value + 1)
      end

      expect(new_attributes.fetch_value(:foo)).to eq(2)
      expect(new_attributes.fetch_value(:bar)).to eq(3)
    end

    specify "comparison for equality is correctly implemented" do
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::Integer.new)
      attributes = builder.build_from_database(foo: "1", bar: "2")
      attributes2 = builder.build_from_database(foo: "1", bar: "2")
      attributes3 = builder.build_from_database(foo: "2", bar: "2")

      expect(attributes2).to eq(attributes)
      expect(attributes3).not_to eq(attributes2)
    end

    it "can be marshalled" do
      builder = AttributeSet::Builder.new(foo: Type::Integer.new, bar: Type::Integer.new)
      attributes = builder.build_from_database(foo: "1", bar: "2")

      marshalled = Marshal.load(Marshal.dump(attributes))

      expect(attributes).to eq(marshalled)
    end

    specify "write_from_database raises on missing attributes" do
      builder = AttributeSet::Builder.new({})
      attributes = builder.build_from_database

      expect {
        attributes.write_from_database(:foo, nil)
      }.to raise_error(ActiveModel::MissingAttributeError)
        .with_message eq("can't write unknown attribute `foo`")
    end

    specify "write_from_user raises on missing attributes" do
      builder = AttributeSet::Builder.new({})
      attributes = builder.build_from_database

      expect {
        attributes.write_from_user(:bar, nil)
      }.to raise_error(ActiveModel::MissingAttributeError)
        .with_message("can't write unknown attribute `bar`")
    end

    specify "modifying frozen attribute set raises" do
      builder = AttributeSet::Builder.new(foo: Type::Value.new)
      attributes = builder.build_from_database(foo: nil)
      attributes.freeze

      expect { attributes.write_from_user(:foo, 1) }.to raise_error(RuntimeError)
      expect { attributes.write_from_database(:foo, 1) }.to raise_error(RuntimeError)
      expect { attributes.write_cast_value(:foo, 1) }.to raise_error(RuntimeError)
      expect { attributes.reset(:foo) }.to raise_error(RuntimeError)
    end
  end
end
