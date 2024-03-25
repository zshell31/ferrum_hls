pub enum TreeNode<L, N> {
    Leaf(L),
    Node(N),
}

pub struct TreeIter<I> {
    stack: Vec<I>,
}

impl<I> TreeIter<I> {
    pub fn new<T>(seed: T) -> Self
    where
        T: IntoIterator<IntoIter = I>,
    {
        Self {
            stack: vec![seed.into_iter()],
        }
    }
}

impl<L, N, I> Iterator for TreeIter<I>
where
    I: Iterator<Item = TreeNode<L, N>>,
    N: IntoIterator<IntoIter = I>,
{
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.last_mut() {
            match node.next() {
                Some(next) => match next {
                    TreeNode::Leaf(leaf) => {
                        return Some(leaf);
                    }
                    TreeNode::Node(node) => {
                        self.stack.push(node.into_iter());
                    }
                },
                None => {
                    self.stack.pop();
                }
            }
        }

        None
    }
}