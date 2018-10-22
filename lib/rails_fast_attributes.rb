require "helix_runtime"
require "rails_fast_attributes/native"
require "rails_fast_attributes/version"
require "active_model"
require "active_model/attribute"
require "active_model/attribute_set"
require "active_model/attribute/user_provided_default"
require "active_record"
require "active_record/relation"
require "active_record/relation/query_attribute"

module RailsFastAttributes
  ORIGINAL_ATTRIBUTE = ActiveModel::Attribute

  class Attribute
    UNINITIALIZED_ORIGINAL_VALUE = Object.new

    def self.inherited(*)
      raise "ActiveModel::Attribute cannot be subclassed when using rails_fast_attributes"
    end

    def self.null(name)
      ORIGINAL_ATTRIBUTE.null(name)
    end

    FromDatabase = self
    FromUser = self
    WithCastValue = self
    Uninitialized = self
    UserProvidedDefault = self

    private_constant :FromDatabase, :FromUser, :WithCastValue, :Uninitialized
  end

  ORIGINAL_ATTRIBUTE.const_get(:Null).class_eval do
    undef with_type
    def with_type(type)
      Attribute.with_cast_value(name, nil, type)
    end
  end

  class AttributeSet
    Builder = RailsFastAttributes::Builder
    YAMLEncoder = ActiveModel::AttributeSet::YAMLEncoder
  end
end

ActiveModel.send(:remove_const, :Attribute)
ActiveModel::Attribute = RailsFastAttributes::Attribute
ActiveModel.send(:remove_const, :AttributeSet)
ActiveModel::AttributeSet = RailsFastAttributes::AttributeSet
