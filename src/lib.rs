use std::rc::*;
use std::cell::*;

pub type RefNode = RefCell<Node>;
pub type RcNode = Rc<RefNode>;
pub type WeakNode = Weak<RefNode>;

pub struct Node {
    parent: Option<WeakNode>,
    children: [Option<RcNode>; 2],
}

pub trait NodeBase: std::ops::Deref<Target = RefNode> {
    fn rc(&self) -> &RcNode;

    fn clone_rc(&self) -> RcNode {
        Rc::clone(self.rc())
    }

    fn parent(&self) -> Option<RcNode> {
        self.rc().borrow().parent.as_ref().and_then(Weak::upgrade)
    }

    fn parent_mut(&self) -> RefMut<Option<WeakNode>> {
        RefMut::map(self.rc().borrow_mut(), |node| &mut node.parent)
    }

    fn child(&self, dir: usize) -> Option<RcNode> {
        assert!(dir < 2);
        Some(Rc::clone(self.rc().borrow().children[dir].as_ref()?))
    }

    fn child_mut(&self, dir: usize) -> RefMut<Option<RcNode>> {
        assert!(dir < 2);
        RefMut::map(self.rc().borrow_mut(), |node| &mut node.children[dir])
    }

    /// 親から見た自分の向き
    fn dir(&self) -> Option<usize> {
        let parent = self.rc().borrow().parent.as_ref()?.upgrade()?;
        for dir in 0 .. 2 {
            if let Some(child) = &parent.borrow().children[dir] {
                if Rc::ptr_eq(self.rc(), child) {
                    return Some(dir);
                }
            }
        }
        // 親が light edge でつながっている場合
        None
    }

    fn is_path_root(&self) -> bool {
        self.dir().is_none()
    }

    fn path_parent(&self) -> Option<RcNode> {
        self.dir().and_then(|_| self.parent())
    }

    fn rotate(&self) {
        if let Some(dir) = self.dir() {
            let parent_weak = self.parent_mut().take().unwrap();
            let parent = parent_weak.upgrade().unwrap();
            let child = self.child_mut(1 ^ dir).take();
            if let Some(child) = child.as_ref() {
                *child.parent_mut() = Some(parent_weak.clone());
            }
            *parent.child_mut(dir) = child.clone();
            if let Some(parent_dir) = parent.dir() {
                let ancestor = parent.parent().unwrap();
                *ancestor.child_mut(parent_dir) = Some(self.clone_rc());
            }
            *self.parent_mut() = parent.parent_mut().replace(Rc::downgrade(self.rc()));
        }
    }

    fn splay(&self) {
        while let Some(parent) = self.path_parent() {
            if parent.is_path_root() {
            } else if self.dir() == parent.dir() {
                parent.rotate();
            } else {
                self.rotate();
            }
            self.rotate();
        }
    }

    /// 自身を木の根のパスにつなげ、そのパスの根にする
    fn expose(&self) {
        self.splay();
        let mut path_root = self.clone_rc();
        while let Some(parent_path) = path_root.parent() {
            *parent_path.child_mut(1) = Some(path_root.clone_rc());
            parent_path.splay();
            path_root = parent_path;
        }
        self.splay();
    }
}

impl NodeBase for RcNode {
    fn rc(&self) -> &Self { self }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
