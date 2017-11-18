require "helix_runtime"
require "rails_fast_attributes/native"
require "rails_fast_attributes/version"
require "active_model"
require "active_model/attribute"
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

    module UserProvidedDefault
      def self.new(name, value, ty, original_attr = nil)
        Attribute.user_provided_default(name, value, ty, original_attr)
      end
    end
  end
end

ActiveModel.send(:remove_const, :Attribute)
ActiveModel::Attribute = RailsFastAttributes::Attribute
ActiveModel::AttributeSet.send(:remove_const, :Builder)
ActiveModel::AttributeSet::Builder = RailsFastAttributes::Builder
