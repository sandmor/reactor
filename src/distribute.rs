use crate::distribute_layout::DistributeLayout;
use orbtk::prelude::*;

widget!(
    /// The `Distribute` defines a layout that is used to stack its children vertical or horizontal.
    ///
    /// **style:** `distribute`
    Distribute {
        /// Margin between widgets in the stack.
        spacing: f64
    }
);

impl Template for Distribute {
    fn template(self, _: Entity, _: &mut BuildContext) -> Self {
        self.name("Distribute").style("distribute")
    }

    fn layout(&self) -> Box<dyn Layout> {
        Box::new(DistributeLayout::new())
    }
}
