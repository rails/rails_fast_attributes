require "helix_runtime"
require "rails_fast_attributes/native"
require "rails_fast_attributes/version"
require "active_record"
require "active_record/attributes"
require "active_record/relation/query_attribute"

module RailsFastAttributes
  ORIGINAL_ATTRIBUTE = ActiveRecord::Attribute
  Attribute::UserProvidedDefault = ORIGINAL_ATTRIBUTE::UserProvidedDefault

  def Attribute.inherited(*)
    raise "ActiveRecord::Attribute cannot be subclassed when using rails_fast_attributes"
  end

  def Attribute.null(name)
    ORIGINAL_ATTRIBUTE.null(name)
  end
end

ActiveRecord::Attribute = RailsFastAttributes::Attribute
ActiveRecord::AttributeSet::Builder = RailsFastAttributes::Builder
