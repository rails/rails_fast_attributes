require "helix_runtime"
require "rails_fast_attributes/native"
require "rails_fast_attributes/version"
require "active_record"
require "active_record/attributes"
require "active_record/relation/query_attribute"

module RailsFastAttributes
  ORIGINAL_ATTRIBUTE = ActiveRecord::Attribute

  class Attribute
    def self.inherited(*)
      raise "ActiveRecord::Attribute cannot be subclassed when using rails_fast_attributes"
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

ActiveRecord::Attribute = RailsFastAttributes::Attribute
ActiveRecord::AttributeSet::Builder = RailsFastAttributes::Builder
