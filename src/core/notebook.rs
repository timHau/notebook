use super::{
    cell::{CellType, LocalValue},
    kernel_client::KernelClientMsg,
};
use crate::core::{cell::Cell, kernel_client::MsgToKernel, topology::Topology};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, sync::mpsc::Sender};
use tracing::info;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct LanguageInfo {
    name: String,
    // version: String,
    file_extension: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NotebookMetadata {
    format_version: String,
}

impl Default for NotebookMetadata {
    fn default() -> Self {
        Self {
            format_version: String::from("0.0.1"),
        }
    }
}

pub type Scope = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notebook {
    pub uuid: String,
    language_info: LanguageInfo,
    meta_data: NotebookMetadata,
    topology: Topology,
    title: String,

    #[serde(skip)]
    pub scope: Scope,

    #[serde(skip)]
    pub kernel_sender: Option<Sender<KernelClientMsg>>,
}

impl Notebook {
    pub fn new(kernel_sender: Sender<KernelClientMsg>) -> Self {
        let mut scope = Scope::default();
        let mut topology = Topology::from_vec(
            vec![
                Cell::new_reactive("def add(a, b):\n  return a + b", &mut scope).unwrap(),
                Cell::new_reactive("import time\n\nfor i in range(20):\n    print(i)\n    time.sleep(1)", &mut scope).unwrap(),
                Cell::new_reactive("a = 1 + 2\nb = 5\nc = 12", &mut scope).unwrap(),
                Cell::new_reactive("add(5, 2)", &mut scope).unwrap(),
                Cell::new_reactive("sum = 0\nfor i in range(10):\n  sum += 1", &mut scope).unwrap(),
                Cell::new_reactive("print(123)", &mut scope).unwrap(),
                Cell::new_reactive(
                    "from torch import nn\nfrom torch.utils.data import DataLoader\nfrom torchvision import datasets\nfrom torchvision.transforms import ToTensor\n\ntraining_data = datasets.FashionMNIST(\n  root='data',\n  train=True,\n  download=True,\n  transform=ToTensor\n)",
                    &mut scope,
                )
                .unwrap(),
                Cell::new_reactive(TMP, &mut scope).unwrap(),
                Cell::new_reactive(
                    "import asyncio\n\nasync def main():\n  print('hello')\n\nasyncio.run(main())",
                    &mut scope,
                ).unwrap(),
            ],
            &mut scope,
        )
        .unwrap();
        topology.build(&mut scope).unwrap();

        let uuid = nanoid!(30);
        Self {
            uuid,
            meta_data: NotebookMetadata::default(),
            scope,
            language_info: LanguageInfo {
                name: String::from("python"),
                // version,
                file_extension: String::from(".py"),
            },
            topology,
            title: String::from("Untitled Notebook"),
            kernel_sender: Some(kernel_sender),
        }
    }

    pub fn eval_cell(&mut self, cell_uuid: &str, next_content: &str) -> Result<(), Box<dyn Error>> {
        // update cell content if it has changed
        self.topology
            .update_cell(cell_uuid, next_content, &mut self.scope)?;

        // get an topological order of the cell uuids and execute them in order
        let execution_seq = self.topology.execution_seq(cell_uuid)?;

        let execution_cells = execution_seq
            .iter()
            .map(|uuid| self.topology.cells.get(uuid).unwrap().clone())
            .collect::<Vec<_>>();

        let locals_of_deps = execution_cells
            .iter()
            .map(|cell| {
                let dependencies = self.topology.get_dependencies(&cell.uuid);
                Self::locals_from_dependencies(&cell, &dependencies)
            })
            .collect::<Vec<_>>();

        let msg = KernelClientMsg::MsgToKernel(MsgToKernel {
            notebook_uuid: self.uuid.clone(),
            cell_uuid: cell_uuid.to_string(),
            locals_of_deps,
            execution_cells,
        });
        let kernel_sender = self.kernel_sender.as_ref().unwrap();
        kernel_sender.send(msg)?;

        Ok(())
    }

    fn locals_from_dependencies(
        cell: &Cell,
        dependencies: &[&Cell],
    ) -> HashMap<String, LocalValue> {
        let mut locals = HashMap::new();
        locals.extend(cell.locals.clone());

        for dependency in dependencies.iter() {
            for (key, value) in dependency.locals.clone().iter() {
                if cell.required.contains(key) {
                    locals.insert(key.clone(), value.clone());
                }
            }
        }
        locals
    }

    pub fn reorder_cells(&mut self, cell_uuids: &[String]) {
        self.topology.reorder_cells(cell_uuids);
    }
}

impl From<&str> for Notebook {
    fn from(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}

const TMP: &str = "
from sklearn.preprocessing import LabelBinarizer
from sklearn.metrics import classification_report
from tensorflow.keras.models import Sequential
from tensorflow.keras.layers import Dense
from tensorflow.keras.optimizers import SGD
from tensorflow.keras.datasets import mnist
from tensorflow.keras import backend as K
import matplotlib.pyplot as plt
import numpy as np
# import argparse


print(\"in main\")

print(\"[INFO] accessing MNIST...\")
((trainX, trainY), (testX, testY)) = mnist.load_data()
trainX = trainX.reshape((trainX.shape[0], 28 * 28 * 1))
testX = testX.reshape((testX.shape[0], 28 * 28 * 1))
# scale data to the range of [0, 1]
trainX = trainX.astype(\"float32\") / 255.0
testX = testX.astype(\"float32\") / 255.0

lb = LabelBinarizer()
trainY = lb.fit_transform(trainY)
testY = lb.transform(testY)

model = Sequential()
model.add(Dense(256, input_shape=(784,), activation=\"sigmoid\"))
model.add(Dense(128, activation=\"sigmoid\"))
model.add(Dense(10, activation=\"softmax\"))

print(\"[INFO] training network...\")
sgd = SGD(0.01)
model.compile(loss=\"categorical_crossentropy\", optimizer=sgd,
                metrics=[\"accuracy\"])
H = model.fit(trainX, trainY, validation_data=(testX, testY),
                epochs=100, batch_size=128)

print(\"[INFO] evaluating network...\")
predictions = model.predict(testX, batch_size=128)
print(classification_report(testY.argmax(axis=1),
                            predictions.argmax(axis=1),
                            target_names=[str(x) for x in lb.classes_]))

plt.figure()
plt.plot(np.arange(0, 100), H.history[\"loss\"], label=\"train_loss\")
plt.plot(np.arange(0, 100), H.history[\"val_loss\"], label=\"val_loss\")
plt.plot(np.arange(0, 100), H.history[\"accuracy\"], label=\"train_acc\")
plt.plot(np.arange(0, 100), H.history[\"val_accuracy\"], label=\"val_acc\")
plt.title(\"Training Loss and Accuracy\")
plt.xlabel(\"Epoch #\")
plt.ylabel(\"Loss/Accuracy\")
plt.legend()
";
