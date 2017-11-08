require "helix_runtime"
require "rails_fast_attributes/native"
require "rails_fast_attributes/version"

module RailsFastAttributes
  def Attribute.inherited(*)
    raise "ActiveRecord::Attribute cannot be subclassed when using rails_fast_attributes"
  end
end

ActiveRecord::Attribute = RailsFastAttributes::Attribute
